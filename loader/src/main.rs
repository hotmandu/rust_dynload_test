use std::{fs, env, path::PathBuf, io::{stdin, stdout, Write}, collections::HashMap, sync::{Arc, RwLock}, str::FromStr};

use abi_stable::{std_types::{RBox, RStr, ROption, RString}, sabi_trait::TD_Opaque};
use loader::{Logger, Logger_TO, MainInterface, MainInterface_TO, BoxedAddonInterface};

#[derive(Clone)]
struct NamedLogger {
    name: String,
}

impl Logger for NamedLogger {
    fn log(&self, msg: RStr) {
        println!("({}) {}", self.name, msg);
    }
}

type AddonToken = RString;
type FnGetAddonInterface = Arc<dyn Fn(&AddonToken, RStr) -> ROption<BoxedAddonInterface<'static>>>;

struct AccessToken {
    token: AddonToken,
    get_addon_interface: FnGetAddonInterface,
}
impl MainInterface for AccessToken {
    fn get_interface_of(&self, addon_name: RStr) -> ROption<BoxedAddonInterface<'static>> {
        (self.get_addon_interface)(&self.token, addon_name)
    }
}

fn main() {
    let mut phase: u32 = 1;

    println!("[{}] Get CWD", phase); phase += 1;

    println!("[ ] current cwd: {}", env::current_dir().unwrap().display());
    let cwd = env::current_exe().unwrap();
    println!("[ ] exe path: {}", cwd.display());
    println!("[ ] set cwd to {}", cwd.parent().unwrap().display());
    env::set_current_dir(cwd.parent().unwrap()).expect("Failed to set CWD");
    println!("[ ] current cwd: {}", env::current_dir().unwrap().display());

    println!("[{}] Scan ./", phase); phase += 1;
    let addon_ext = env::consts::DLL_EXTENSION;
    println!("[ ] scan for ext: {}", addon_ext);
    
    let paths = fs::read_dir("./").unwrap();

    let mut dlls: Vec<PathBuf> = vec![];

    for path in paths {
        let buf = path.unwrap().path();
        if buf.extension().map_or(false, |p| p.eq(addon_ext)) {
            println!("[ ] found '{}'", buf.display());
            dlls.push(buf);
        }
    }

    println!("[{}] Load .{}'s & Fetch ID's", phase, addon_ext); phase += 1;
    let mut addon_base_list = HashMap::<String, loader::AddonObject_Ref>::new();
    for dll_path in dlls {
        println!("[ ] target: {}", dll_path.display());
        let load_result = abi_stable::library::lib_header_from_path(&dll_path)
            .and_then(|x| x.init_root_module::<loader::AddonObject_Ref>());
        // Doesn't work from second dll load.
        // https://github.com/rodrimati1992/abi_stable_crates/issues/92
        // let load_result = loader::AddonObject_Ref::load_from_file(&dll_path);
        match load_result {
            Ok(addon_base) => {
                let name = addon_base.name();
                let version = addon_base.version();
                println!("[+] loaded '{}' v{}", name, version);
                let deps = addon_base.dependency();
                if deps.len() > 0 {
                    println!("[ ]  required deps: {:?}", deps.as_slice());
                }
                addon_base_list.insert(name.to_string(), addon_base);
            },
            Err(err) => {
                println!("[-] failed: {}", err);
            }
        }
    }
    
    println!("[{}] Call addons' on_load(), Solve deps", phase); phase += 1;
    let addons_cnt = addon_base_list.len();
    let addon_list = HashMap::<String, loader::Addon_TO<'static, RBox<()>>>::new();
    let addon_list = Arc::from(RwLock::from(addon_list));
    let al_tr = Arc::clone(&addon_list);
    let fn_get_addon_interface: FnGetAddonInterface = Arc::new(move |_token, target| {
        // let permitted = todo!()
        let al_tr_b = al_tr.read().unwrap();
        let lkup = al_tr_b.get(target.as_str()).and_then(|v| Some(v.get_interface()));
        ROption::from(lkup)
    });
    let result = loop {
        
        let al_b = addon_list.read().unwrap();
        let al_len = al_b.len();
        drop(al_b);
        if addons_cnt == al_len {
            break true;
        }

        for (name, addon_ref) in addon_base_list.iter() {
            // Load only if not loaded before
            if addon_list.read().unwrap().contains_key(name) { continue; }

            let deps = addon_ref.dependency();
            let al_b = addon_list.read().unwrap();
            let deps_ok = deps.iter().all(|dep| al_b.contains_key(dep.as_str()));
            drop(al_b);
            if deps_ok {
                println!("[ ] call '{}'.on_load()", name);
                let named_logger = NamedLogger { name: name.clone() };
                let mut addon = addon_ref.new()(Logger_TO::from_value(named_logger, TD_Opaque));
                let at_to = MainInterface_TO::from_value(AccessToken { token: RString::from_str(name).unwrap(), get_addon_interface: Arc::clone(&fn_get_addon_interface) }, TD_Opaque);
                addon.on_load(at_to);
                let mut al_mb = addon_list.write().unwrap();
                al_mb.insert(name.clone(), addon);
            }
        }

        let al_b = addon_list.read().unwrap();
        let al_len_2 = al_b.len();
        drop(al_b);
        if al_len == al_len_2 {
            println!("[!] Cannot solve deps graph. Exit program.");
            break false;
        }
    };

    if !result {
        return;
    }

    println!("[{}] Begin issue loop, submit empty id to break", phase);
    loop {
        let mut raw_input = String::new();
        print!(" >  ");
        stdout().flush().expect("Failed to flush stdout");
        stdin().read_line(&mut raw_input).expect("Failed to get input");
        let input = raw_input.trim();
        if input == "" {
            break;
        } else {
            // TODO: addon_list.write() inside issue() will cause a deadlock... How to fix it?
            // Why should I fix this?
            //  - addon_list.write() is called whenever a plugin tries to load another plugin.
            //    But why a plugin loads other plugin at first?
            let al_b = addon_list.read().unwrap();
            let res = al_b.get(input);
            if let Some(addon) = res {
                println!("[=] Launch module '{}'", input);
                let res = addon.issue();
                println!("[=] Result: {:?}, Msg: {}", res.state, res.msg);
            } else {
                println!("[=] Module '{}' not found", input);
            }
        }
    }

    println!("[!] Exit program");

}
