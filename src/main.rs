use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use iced::widget::{button, column, container, row, text, text_editor};
use iced::{executor, Application, Command, Element, Font, Settings, Theme};

fn main() -> Result<(), iced::Error> {
    Editor::run(Settings {
        fonts: vec![include_bytes!("../fonts/nixary-icons.ttf")
            .as_slice()
            .into()],
        ..Settings::default()
    })
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IOFailed(io::ErrorKind),
}

#[derive(Debug, Clone)]
enum Message {
    New,
    Edit(text_editor::Action),
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    FileSaved(Result<PathBuf, Error>),
    Open,
    Save,
}
struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
}

impl Application for Editor {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                path: None,
                content: text_editor::Content::with_text(include_str!("main.rs")),
                error: None,
            },
            Command::perform(load_file(default_file()), Message::FileOpened),
        )
    }

    fn title(&self) -> String {
        String::from("Nixary Editor")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::Edit(action) => {
                self.content.perform(action);
                self.error = None;
                Command::none()
            }
            Message::Open => Command::perform(browse_file(), Message::FileOpened),
            Message::New => {
                self.path = None;
                self.content = text_editor::Content::new();
                Command::none()
            }
            Message::Save => {
                let content_text = self.content.text();
                Command::perform(
                    save_file(self.path.clone(), content_text),
                    Message::FileSaved,
                )
            }
            Message::FileOpened(Ok((path, content))) => {
                self.path = Some(path);
                self.content = text_editor::Content::with_text(&content);
                Command::none()
            }
            Message::FileOpened(Err(err)) => {
                self.error = Some(err);
                Command::none()
            }
            Message::FileSaved(Ok(path)) => {
                self.path = Some(path);
                Command::none()
            }
            Message::FileSaved(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            action(new_icon(), Message::New),
            action(open_icon(), Message::Open),
            action(save_icon(), Message::Save),
        ]
        .spacing(5);
        let input = text_editor(&self.content).on_action(Message::Edit);

        let status_bar = {
            let status = if let Some(Error::IOFailed(error)) = self.error.as_ref() {
                text(error.to_string())
            } else {
                match self.path.as_deref().and_then(Path::to_str) {
                    Some(path) => text(path).size(13),
                    None => text("New File"),
                }
            };

            let position = {
                let (line, column) = self.content.cursor_position();
                text(format!("{}:{}", line + 1, column + 1))
            };

            row![status, position]
        };

        container(column![controls, input, status_bar].spacing(10))
            .padding(10)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dracula
    }
}

fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}

async fn save_file(path: Option<PathBuf>, content_text: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .set_title("Choose a file name..")
            .save_file()
            .await
            .ok_or(Error::DialogClosed)
            .map(|handle| handle.path().to_owned())?
    };
    tokio::fs::write(&path, content_text)
        .await
        .map_err(|error| Error::IOFailed(error.kind()))?;

    Ok(path)
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
        .map_err(Error::IOFailed)?;
    Ok((path, contents))
}

async fn browse_file() -> Result<(PathBuf, Arc<String>), Error> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Browse a file")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;
    load_file(handle.path().to_owned()).await
}

fn action<'a>(content: Element<'a, Message>, on_press: Message) -> Element<'a, Message> {
    button(container(content).width(20).center_x())
        .on_press(on_press)
        .padding([5, 8])
        .into()
}

fn new_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{F0F6}')
}

fn save_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{E800}')
}

fn open_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{F115}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("nixary-icons");

    text(codepoint).font(ICON_FONT).into()
}
