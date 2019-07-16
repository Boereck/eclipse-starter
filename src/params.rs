
/// This struct holds all information needed by the launcher executable
/// that needs to be parsed from the command line and configuration ini file.
#[derive(Debug, Default)]
pub struct EclipseLauncherParams {
    pub name: Option<String>,
    pub eclipse_library: Option<String>,
    pub suppress_errors: bool,
    pub protect: Option<String>,
    pub launcher_ini: Option<String>,
    pub vm_args: Option<Vec<String>>,
}