
#[derive(Debug, Default)]
pub struct EclipseLauncherParams {
    pub name: Option<String>,
    pub library: Option<String>,
    pub suppress_errors: bool,
    pub protect: Option<String>,
    pub launcher_ini: Option<String>,
    pub vm_args: Option<Vec<String>>,
}