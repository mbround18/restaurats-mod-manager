use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct BepInExPoller {
    game_dir: PathBuf,
    ready_flag: Arc<Mutex<bool>>,
}

impl BepInExPoller {
    pub fn new(game_dir: PathBuf) -> Self {
        Self {
            game_dir,
            ready_flag: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(self) -> Arc<Mutex<bool>> {
        let flag = Arc::clone(&self.ready_flag);
        let game_dir = self.game_dir.clone();

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(5));
                if crate::bepinex::is_bep_installed(&game_dir) {
                    *flag.lock().unwrap() = true;
                    break;
                }
            }
        });

        Arc::clone(&self.ready_flag)
    }
}
