use winrt_notification::{Toast, Duration, Sound};


pub fn app_is_running() {
    Toast::new(Toast::POWERSHELL_APP_ID)
        .duration(Duration::Short)
        .text1("SilentKeys is running.")
        .sound(Some(Sound::SMS))
        .show()
        .expect("unable to toast")
}

pub fn app_is_exiting() {
    Toast::new(Toast::POWERSHELL_APP_ID)
        .duration(Duration::Short)
        .text1("SilentKeys is exiting.")
        .sound(Some(Sound::SMS))
        .show()
        .expect("unable to toast")
}
