//! Registers Bahia's logon task through the Task Scheduler API (what
//! `autostart::set` now does) for the given bahia.exe, and reports.
//! `spike_autostart <path to bahia.exe>` or `spike_autostart off`.

fn main() -> anyhow::Result<()> {
    let arg = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("usage: spike_autostart <bahia.exe path | off>"))?;
    let target = (arg != "off").then_some(arg.as_str());
    match bahia_shell::autostart::set(target) {
        Ok(()) => println!("ok; is_enabled = {}", bahia_shell::autostart::is_enabled()),
        Err(error) => return Err(error.into()),
    }
    Ok(())
}
