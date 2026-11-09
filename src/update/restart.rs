use std::process::Command;

use anyhow::Result;

pub fn restart() -> Result<()> {
    let exe = std::env::current_exe()?;
    let args: Vec<_> = std::env::args_os().skip(1).collect();

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = Command::new(exe).args(args).exec();
        Err(err.into())
    }

    #[cfg(windows)]
    {
        Command::new(exe).args(args).spawn()?;
        Ok(())
    }
}
