use anyhow::Result;

pub use bundle::{Options, run_bundle};

mod bundle;

pub type ModulePath = String;
pub type ModuleSource = String;

pub trait ModuleLoader {
    fn load(&self, specifier: &str) -> Result<ModuleSource>;
    fn resolve(&self, base: Option<&str>, specifier: &str) -> Result<ModulePath>;
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn test_run_bundle() -> Result<()> {
        let ret = run_bundle("fixtures/main.ts", &Default::default())?;
        assert_eq!(ret, "(function(){async function execute(name){console.log(\"Executing lib\");return`Hello ${name}!`;}async function main(){console.log(\"Executing main\");console.log(await execute(\"world\"));}return{default:main};})();");

        Ok(())
    }
}
