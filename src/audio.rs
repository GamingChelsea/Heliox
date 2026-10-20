use std::{fs::File, io::BufReader, path::PathBuf};

pub enum AudioCommand {
    Play(PathBuf),
    ResumePause,
    Stop,
}

pub fn init_audio() -> std::sync::mpsc::Sender<AudioCommand> {
    let (sender, receiver) = std::sync::mpsc::channel::<AudioCommand>();

    std::thread::spawn(move || {
        let mut sink_handle =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
        sink_handle.log_on_drop(false);

        let mut current_player = None;

        while let Ok(command) = receiver.recv() {
            match command {
                AudioCommand::Play(path) => {
                    // todo: Improve later maybe only data not a path as input for streaming
                    let file = BufReader::new(File::open(path).unwrap());
                    current_player = Some(rodio::play(&sink_handle.mixer(), file));
                }
                AudioCommand::ResumePause => {
                    if let Some(ref player) = current_player {
                        if let Ok(plyer) = player {
                            if plyer.is_paused() {
                                plyer.play();
                            } else {
                                plyer.stop();
                            }
                        }
                    }
                }
                AudioCommand::Stop => {
                    current_player = None;
                }
            }
        }
    });

    sender
}
