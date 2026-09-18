use yasal::core::plugin::Plugin;
use yasal::plugins::calculator::CalculatorPlugin;
use yasal::plugins::shell_command::ShellCommandPlugin;
use yasal::plugins::system_commands::SystemCommandsPlugin;
use yasal::plugins::web_search::WebSearchPlugin;

#[test]
fn test_calculator_plugin() {
    let calc = CalculatorPlugin::new();
    let res = calc.query("= 12 * 8");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].title, "= 96");
    assert_eq!(res[0].category, "Kalkulator");
}

#[test]
fn test_web_search_plugin() {
    let web = WebSearchPlugin::new("Google");
    let res = web.query("?? rust programming");
    assert_eq!(res.len(), 1);
    assert!(res[0].title.contains("rust programming"));

    let url_res = web.query("https://github.com");
    assert_eq!(url_res.len(), 1);
    assert!(url_res[0].title.contains("https://github.com"));
}

#[test]
fn test_shell_command_plugin() {
    let shell = ShellCommandPlugin::new();
    let res = shell.query("> ipconfig /all");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].title, "> ipconfig /all");
}

#[test]
fn test_system_commands_plugin() {
    let sys = SystemCommandsPlugin::new();
    let res = sys.query("lock");
    assert!(!res.is_empty());
    assert!(res[0].title.contains("Lock Screen"));
}
