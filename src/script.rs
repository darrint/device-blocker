use table::Table;

pub fn write_script(table : &Table, old_chain: Option<&String>, new_chain: &String, dest: &mut String) {
    dest.push_str(&format!("iptables -N {}\n", new_chain));
    for entry in &table.entries {
        dest.push_str(&format!("iptables -A {} -m mac --mac-source {} -j DROP\n", new_chain, entry.mac));
    }
    dest.push_str(&format!("iptables -I FORWARD 0 -j {}\n", new_chain));
    if let Some(old) = old_chain {
        dest.push_str(&format!("iptables -D FORWARD -j {}\n", old));
    }

}


