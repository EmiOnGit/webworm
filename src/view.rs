use iced::alignment::Horizontal;
use iced::theme::{self};
use iced::widget::text::LineHeight;
use iced::widget::{
    button, column, container, image, row, text, text_input, Column, Image, Row, Space,
};
use iced::Length;
use iced::{Alignment, Element};

use crate::bookmark::{Bookmark, Poster, SyncMode};
use crate::filter::Filter;
use crate::gui::{FONT_SIZE, FONT_SIZE_HEADER};
use crate::icons::{Icon, ICON_SIZE};
use crate::id::MovieId;
use crate::message::{BookmarkMessage, LinkMessage, Message, ShiftPressed};
use crate::movie::TmdbMovie;
use crate::movie_details::{Episode, EpisodeDetails, MovieDetails};
use crate::state::{InputCaches, InputKind};

impl Bookmark {
    pub fn card_view<'a>(
        &'a self,
        details: Option<&MovieDetails>,
        poster: Option<&'a Poster>,
    ) -> Element<Message> {
        let picture_row = row![
            Space::with_width(Length::Fill),
            picture_button(self.movie.id, poster, Length::FillPortion(3)),
            button(Icon::Play.svg())
                .on_press(Message::LinkMessage(
                    self.movie.id,
                    LinkMessage::LinkToClipboard(details.cloned(), ShiftPressed::Unknown)
                ))
                .width(Length::Fixed(ICON_SIZE * 2.)),
        ];
        let progress = text(format!("PROGRESS: {}", self.current_episode.as_info_str()))
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);
        let latest = if let Some(details) = &details {
            if let Some(last) = details.last_published() {
                text(format!(
                    "LATEST: {}",
                    Into::<Episode>::into(last.episode).as_info_str()
                ))
            } else {
                text("no latest episode")
            }
        } else {
            text("details not loaded")
        }
        .width(Length::Fill)
        .horizontal_alignment(Horizontal::Center);
        column![
            picture_row,
            text(self.movie.name.as_str())
                .horizontal_alignment(Horizontal::Center)
                .width(Length::Fill)
                .size(FONT_SIZE_HEADER),
            progress,
            latest
        ]
        .into()
    }
}
impl TmdbMovie {
    pub fn view<'a>(&'a self, poster: Option<&'a Poster>) -> Element<Message> {
        let info_column = column![
            text(self.name.as_str()).style(theme::Text::Default),
            text(format!("Rating: {0}%", self.rating())),
        ];
        let description: String = self
            .overview
            .split_whitespace()
            .take(30)
            .collect::<Vec<&str>>()
            .join(" ");
        iced::widget::container(
            row![
                picture_view(poster, Length::FillPortion(3)),
                info_column.width(Length::FillPortion(2)),
                text(description + "...")
                    .width(Length::FillPortion(6))
                    .style(theme::Text::Default),
                button(Icon::Plus.svg())
                    .on_press(Message::AddBookmark(self.id))
                    .padding(10)
                    .width(Length::FillPortion(1))
            ]
            .spacing(20)
            .align_items(Alignment::Start),
        )
        .style(theme::Container::Transparent)
        .width(Length::Fill)
        .padding(20.)
        .into()
    }
}
fn picture_button(id: MovieId, poster: Option<&Poster>, width: Length) -> Element<Message> {
    if let Some(Poster::Image(img)) = &poster {
        button(Image::<image::Handle>::new(img.clone())).width(width)
    } else {
        button("IMG").width(width)
    }
    .on_press(Message::FilterChanged(Filter::Details(id)))
    .into()
}
fn picture_view(poster: Option<&Poster>, width: Length) -> Element<Message> {
    if let Some(Poster::Image(img)) = &poster {
        container(Image::<image::Handle>::new(img.clone())).width(width)
    } else {
        container("IMG").width(width)
    }
    .into()
}
pub(crate) fn view_details(
    movie: &TmdbMovie,
    input_caches: &InputCaches,
    details: Option<&MovieDetails>,
    poster: Option<&Poster>,
    current: Option<EpisodeDetails>,
    sync: SyncMode,
) -> Element<'static, Message> {
    column![
        button(Icon::ArrowLeft.svg())
            .on_press(Message::FilterChanged(Filter::Bookmarks))
            .width(Length::Fixed(ICON_SIZE * 2.)),
        row![
            text(&movie.name).size(FONT_SIZE_HEADER),
            text(format!(" [{}]", &movie.original_name)).size(FONT_SIZE_HEADER)
        ],
        details_view_info(details, poster, current.as_ref()),
        details_view_edit(input_caches, movie.id, sync)
    ]
    .spacing(20)
    .into()
}
fn details_view_edit(
    input_caches: &InputCaches,
    id: MovieId,
    sync: SyncMode,
) -> Column<'static, Message> {
    let episode = &input_caches[InputKind::EpisodeInput];
    let season = &input_caches[InputKind::SeasonInput];
    let current_progress_row = row![
        text("Progress ").size(FONT_SIZE),
        // Space::with_width(Length::Fixed(20.)),
        column![
            text("Episode").line_height(LineHeight::Relative(2.)),
            text("Season").line_height(LineHeight::Relative(2.)),
        ],
        column![
            text_input("0", episode)
                .on_submit(Message::InputSubmit(InputKind::EpisodeInput))
                .on_input(|input| { Message::InputChanged(InputKind::EpisodeInput, input) })
                .width(Length::Fixed(60.)),
            text_input("0", season)
                .on_submit(Message::InputSubmit(InputKind::SeasonInput))
                .on_input(|input| { Message::InputChanged(InputKind::SeasonInput, input) })
                .width(Length::Fixed(60.))
        ],
        match sync {
            SyncMode::NoSync => {
                button(Icon::Unlock.svg())
            }
            SyncMode::Tmdb => {
                button(Icon::Lock.svg())
            }
        }
        .width(Length::Fixed(50.))
        .on_press(Message::BookmarkMessage(id, BookmarkMessage::ToggleSync))
    ]
    .spacing(10);
    let link = &input_caches[InputKind::LinkInput];
    let link_input = text_input("https://link_to_movie/episode-{e}-season-{s}}", link)
        .on_submit(Message::InputSubmit(InputKind::LinkInput))
        .on_input(|input| Message::InputChanged(InputKind::LinkInput, input))
        .width(Length::Fill);
    let enter = button(Icon::Check.svg())
        .on_press(Message::InputSubmit(InputKind::LinkInput))
        .width(Length::Fixed(50.));
    let link_row = row![text("URL ").size(FONT_SIZE), link_input, enter].spacing(10);

    let remove_bookmark = row![
        text("Delete Bookmark").size(FONT_SIZE),
        button(Icon::Trash.svg())
            .on_press(Message::RemoveBookmark(id))
            .width(Length::Fixed(50.))
    ]
    .spacing(10);
    column![current_progress_row, link_row, remove_bookmark]
}
fn details_view_info(
    details: Option<&MovieDetails>,
    poster: Option<&Poster>,
    current: Option<&EpisodeDetails>,
) -> Row<'static, Message> {
    let mut poster_row = Row::new();
    if let Some(poster) = poster {
        let Poster::Image(image) = poster;
        poster_row = poster_row
            .push(Image::<image::Handle>::new(image.clone()).width(Length::FillPortion(1)));
    }
    poster_row = poster_row.push(Space::with_width(Length::Fixed(20.)));
    let latest_episode_block: Element<_, _, _> = if let Some(details) = details {
        let latest = details.last_published().unwrap();
        row![
            container(text("Latest Episode: ").size(FONT_SIZE)).center_y(),
            column![text(&latest.name), text(latest.episode.as_info_str())]
        ]
        .into()
    } else {
        Space::with_width(Length::FillPortion(1)).into()
    };
    let upcoming_episode_block: Element<_, _, _> = if let Some(details) = details {
        if let Some(upcoming_episode) = details.next_episode_to_air() {
            row![
                text("Upcoming Episode: ").size(FONT_SIZE),
                column![
                    text(&upcoming_episode.name),
                    text(upcoming_episode.episode.as_info_str()),
                    text(format!(
                        "Releases at {}",
                        upcoming_episode.air_date.unwrap_or("---".into())
                    ))
                ]
            ]
            .into()
        } else {
            Space::with_width(Length::FillPortion(1)).into()
        }
    } else {
        Space::with_width(Length::FillPortion(1)).into()
    };
    let details_block = column![
        row![
            text("Current Episode: ").size(FONT_SIZE),
            if let Some(current) = current {
                column![text(&current.name), text(current.episode.as_info_str())]
            } else {
                column![text("TITLE"), text("---")]
            },
        ],
        latest_episode_block,
        upcoming_episode_block
    ]
    .spacing(20)
    .width(Length::FillPortion(4));
    poster_row = poster_row.push(details_block);
    poster_row
}
