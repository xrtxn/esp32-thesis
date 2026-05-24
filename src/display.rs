use core::range::RangeInclusive;

use embedded_graphics::geometry::Size;
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::prelude::{Dimensions, DrawTarget, OriginDimensions, Point};
use embedded_graphics::prelude::{Drawable, Primitive};
use embedded_graphics::primitives::{
    Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, RoundedRectangle,
};
use embedded_graphics::text::Text;
use heapless::format as hformat;
use weact_studio_epd::Color as EpdColor;

#[cfg(feature = "defmt")]
use crate::defmt::info;

#[allow(dead_code)]
pub const DISPLAY_WIDTH: u32 = 300;
pub const DISPLAY_HEIGHT: u32 = 400;
const EXTRA_BOTTOM_SPACE: i32 = 15;
const EVENT_FONT: MonoFont = profont::PROFONT_10_POINT;
const MINI_FONT: MonoFont = profont::PROFONT_7_POINT;
const START_POS: i32 = calculate_text_width(5, EVENT_FONT) as i32;
const CHARACTER_STYLE: MonoTextStyle<'static, EpdColor> =
    MonoTextStyle::new(&EVENT_FONT, FOREGROUND_COLOR);
const MINI_CHARACTER_STYLE: MonoTextStyle<'static, EpdColor> =
    MonoTextStyle::new(&MINI_FONT, FOREGROUND_COLOR);
pub const BACKGROUND_COLOR: EpdColor = EpdColor::White;
pub const FOREGROUND_COLOR: EpdColor = EpdColor::Black;
const DRAW_NEW_DAY: bool = true;

const OVERWRITE_STYLE: PrimitiveStyle<EpdColor> = PrimitiveStyleBuilder::new()
    .fill_color(BACKGROUND_COLOR)
    .stroke_color(FOREGROUND_COLOR)
    .stroke_width(1)
    .build();

const BORDERLESS_OVERWRITE_STYLE: PrimitiveStyle<EpdColor> =
    PrimitiveStyle::with_fill(BACKGROUND_COLOR);

const fn calculate_row_padding(start_hour: u8, end_hour: u8) -> i32 {
    assert!(
        end_hour > start_hour,
        "End hour must be greater than start hour"
    );
    let text_height = EVENT_FONT.character_size.height as i32;
    let item_count = (end_hour - start_hour) as i32;
    let total_text_size = text_height * item_count;
    let remaining_space = DISPLAY_HEIGHT as i32 - EXTRA_BOTTOM_SPACE - total_text_size;
    remaining_space / (item_count)
}

pub static DISPLAY_HOURS: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(8);
pub static SHOW_CURRENT_DAY_ONLY: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

pub fn limit_to_today() -> bool {
    SHOW_CURRENT_DAY_ONLY.load(core::sync::atomic::Ordering::Relaxed)
}

pub fn get_display_hours() -> u8 {
    if DRAW_NEW_DAY {
        DISPLAY_HOURS.load(core::sync::atomic::Ordering::Relaxed) - 1
    } else {
        DISPLAY_HOURS.load(core::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(debug_assertions)]
pub(crate) fn add_footer_info<D>(display: &mut D)
where
    D: DrawTarget<Color = EpdColor> + OriginDimensions,
    D::Error: core::fmt::Debug,
{
    use embedded_graphics::text::Text;

    let git_commit = env!("GIT_SHORT");
    let git_dirty: bool = env!("GIT_DIRTY").parse().unwrap_or(false);
    // 8 is the text + 8 is short hash in build.rs + 1 is possible *
    let mut build_info: heapless::String<17> = hformat!("commit: {git_commit}").unwrap();
    if git_dirty {
        build_info.push_str("*").unwrap();
    }

    let text_style = embedded_graphics::text::TextStyleBuilder::new()
        .alignment(embedded_graphics::text::Alignment::Right)
        .baseline(embedded_graphics::text::Baseline::Bottom)
        .build();

    let etext = Text::with_text_style(
        &build_info,
        display.bounding_box().bottom_right().unwrap(),
        MINI_CHARACTER_STYLE,
        text_style,
    );

    let mut text_bb = etext.bounding_box();
    text_bb.size.width += 2;
    text_bb.size.height += 1;
    text_bb.top_left.x -= 1;

    text_bb.into_styled(OVERWRITE_STYLE).draw(display).unwrap();

    etext.draw(display).unwrap();
}

pub(crate) fn draw_time_row_header<D>(
    display: &mut D,
    start_display_hour: u8,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = EpdColor> + OriginDimensions,
{
    let end_hour = start_display_hour + get_display_hours();
    let text_height = EVENT_FONT.character_size.height as i32;
    let row_padding = calculate_row_padding(start_display_hour, end_hour);

    let y_step = text_height + row_padding;
    let start_pos = display.bounding_box().top_left;

    let mut y_offset = 0;
    let mut drawn_day = false;

    for hour in start_display_hour..=end_hour {
        let rel_hour = hour % 24;

        // Add the extra gap for the new day, but do NOT skip the loop.
        // This pushes the 00:00 text down by one extra row.
        if rel_hour == 0 && !drawn_day && DRAW_NEW_DAY {
            drawn_day = true;
            y_offset += y_step;
        }

        let fmt_hour: heapless::String<5> = hformat!("{:0>2}:00", rel_hour).unwrap();

        Text::with_baseline(
            &fmt_hour,
            start_pos + Point::new(0, y_offset),
            CHARACTER_STYLE,
            embedded_graphics::text::Baseline::Top,
        )
        .draw(display)?;

        y_offset += y_step;
    }

    Ok(())
}

/// Calculates the starting position of the event based on the screen size
fn calculate_start_height(start_minute: u16, start_display_hour: u8) -> u32 {
    let text_height = EVENT_FONT.character_size.height as i32;

    let row_padding =
        calculate_row_padding(start_display_hour, start_display_hour + get_display_hours());
    let one_hour_height = text_height + row_padding;
    let one_minute = one_hour_height as f32 / 60.0;

    (one_minute * start_minute as f32) as u32 + (text_height / 2) as u32
}

/// Calculates the ending position of the event based on the screen size
fn calculate_end_height(end_minute: u16, start_display_hour: u8) -> u32 {
    let text_height = EVENT_FONT.character_size.height as i32;

    let row_padding =
        calculate_row_padding(start_display_hour, start_display_hour + get_display_hours());
    let one_hour_height = text_height + row_padding;
    let one_minute = one_hour_height as f32 / 60.0;

    (one_minute * end_minute as f32) as u32 + (text_height / 2) as u32
}

const fn calculate_text_width(char_count: u16, font: MonoFont) -> u16 {
    if char_count == 0 {
        return 0;
    }

    (char_count * font.character_size.width as u16)
        + ((char_count - 1) * font.character_spacing as u16)
        + font.character_size.width as u16 / 2
}

pub(crate) fn draw_sync_time<D>(display: &mut D, time: &jiff::Zoned)
where
    D: DrawTarget<Color = EpdColor> + OriginDimensions,
    D::Error: core::fmt::Debug,
{
    #[cfg(feature = "defmt")]
    crate::defmt::info!("Calendar sync time: {}", crate::defmt::Debug2Format(&time));
    let fmt_time: heapless::String<11> =
        hformat!("Sync: {:02}:{:02}", time.hour(), time.minute()).unwrap();

    let text_style = embedded_graphics::text::TextStyleBuilder::new()
        .alignment(embedded_graphics::text::Alignment::Right)
        .baseline(embedded_graphics::text::Baseline::Top)
        .build();

    let pos = Point::new(
        display.bounding_box().bottom_right().unwrap().x,
        display.bounding_box().top_left.y,
    );

    let etext = Text::with_text_style(&fmt_time, pos, MINI_CHARACTER_STYLE, text_style);

    let mut text_bb = etext.bounding_box();
    extend_rectangle(&mut text_bb);

    text_bb
        .bounding_box()
        .into_styled(BORDERLESS_OVERWRITE_STYLE)
        .draw(display)
        .unwrap();

    etext.draw(display).unwrap();
}

pub fn fill_rectangle_with_diagonal<D>(display: &mut D, rect: Rectangle)
where
    D: DrawTarget<Color = EpdColor> + OriginDimensions,
    D::Error: core::fmt::Debug,
{
    rect.into_styled(OVERWRITE_STYLE).draw(display).unwrap();

    let mut bottom_left = Point {
        x: rect.top_left.x,
        y: rect.bottom_right().unwrap().y,
    };

    fn move_right(point: &mut Point) {
        point.x += 20;
    }

    let mut top_point = rect.top_left;
    top_point.x += rect.size.height as i32 / 2;
    loop {
        Line::new(bottom_left, top_point)
            .into_styled(PrimitiveStyle::with_stroke(FOREGROUND_COLOR, 1))
            .draw(display)
            .unwrap();

        if top_point.x >= rect.bottom_right().unwrap().x {
            break;
        }

        move_right(&mut bottom_left);
        move_right(&mut top_point);
    }
}

pub fn draw_base_calendar<D>(display: &mut D, start_display_hour: u8, time: &jiff::Zoned)
where
    D: DrawTarget<Color = EpdColor> + OriginDimensions,
    D::Error: core::fmt::Debug,
{
    // `00:00` + 1 is because of the left side time text
    const HOUR_LENGTH: u8 = 5 + 1;

    let text_height = EVENT_FONT.character_size.height as i32;
    let text_width = EVENT_FONT.character_size.width as i32;

    let position = display.bounding_box().top_left;

    let mut drawn_day = false;

    let end_hour = start_display_hour + get_display_hours();
    let row_padding = calculate_row_padding(start_display_hour, end_hour);
    let y_step = text_height + row_padding;
    let mut y_offset = 0;

    for hour in start_display_hour..=end_hour {
        let rel_hour = hour % 24;
        let start_pos =
            position + Point::new(text_width * HOUR_LENGTH as i32, y_offset + text_height / 2);
        let finish_pos =
            position + Point::new(display.size().width as i32, y_offset + text_height / 2);

        if rel_hour == 0 && !drawn_day {
            use icu::locale::locale;
            use icu_datetime::DateTimeFormatter;

            let icu_date = icu_datetime::input::Date::try_new_gregorian(
                time.year() as i32,
                time.month() as u8,
                time.day() as u8,
            )
            .unwrap();

            let locale = locale!("hu-HU");
            let formatter =
                DateTimeFormatter::try_new(locale.into(), icu_datetime::fieldsets::YMDE::long())
                    .unwrap();

            let str = formatter.format(&icu_date).to_string();

            let mut rect_start = start_pos;
            rect_start.x = -20;

            let mut rect_end = finish_pos;
            rect_end.y += y_step;

            fill_rectangle_with_diagonal(display, Rectangle::with_corners(rect_start, rect_end));

            Text::with_baseline(
                &str,
                position + Point::new(text_width * HOUR_LENGTH as i32, y_offset + y_step / 2),
                CHARACTER_STYLE,
                embedded_graphics::text::Baseline::Top,
            )
            .draw(display)
            .unwrap();

            drawn_day = true;
        };

        Line::new(start_pos, finish_pos)
            .into_styled(PrimitiveStyle::with_stroke(FOREGROUND_COLOR, 1))
            .draw(display)
            .unwrap();
        y_offset += y_step;
    }
    // height is at max
}

pub(crate) fn draw_time_ticker<D>(display: &mut D, time: &jiff::Zoned, start_display_hour: u8)
where
    D: DrawTarget<Color = EpdColor> + OriginDimensions,
    D::Error: core::fmt::Debug,
{
    let x = START_POS;
    let end_x = START_POS + 40;

    let display_hours = get_display_hours();
    if time.hour() < start_display_hour as i8
        || time.hour() > (start_display_hour + display_hours) as i8
    {
        #[cfg(feature = "defmt")]
        crate::defmt::warn!(
            "Current time {} is out of display bounds ({}-{}), skipping time ticker",
            crate::defmt::Debug2Format(&time),
            start_display_hour,
            start_display_hour + display_hours
        );
        return;
    }

    let y = calculate_start_height(
        date_to_mins(time) - start_display_hour as u16 * 60,
        start_display_hour,
    );

    let end_y = y;

    Line::new(Point::new(x, y as i32), Point::new(end_x, end_y as i32))
        .into_styled(PrimitiveStyle::with_stroke(FOREGROUND_COLOR, 1))
        .draw(display)
        .unwrap();
}

pub(crate) struct OccupiedSpaces(alloc::vec::Vec<OccupiedSpace>);

impl OccupiedSpaces {
    pub fn new() -> Self {
        Self(alloc::vec::Vec::new())
    }
}

impl OccupiedSpaces {
    fn add_space(
        &mut self,
        y_range: core::range::RangeInclusive<u16>,
        x_range: core::range::RangeInclusive<u16>,
    ) {
        self.0.push(OccupiedSpace { y_range, x_range });
    }

    fn get_next_free_slot(
        &self,
        new_event_range: core::range::RangeInclusive<u16>,
        width: u16,
    ) -> u16 {
        let mut overlapping: alloc::vec::Vec<&OccupiedSpace> = self
            .0
            .iter()
            .filter(|occ_space| {
                occ_space.y_range.start < new_event_range.last
                    && occ_space.y_range.last > new_event_range.start
            })
            .collect();

        overlapping.sort_by_key(|s| s.x_range.start);

        let mut current_x = 0;
        for space in overlapping {
            // If there is a gap large enough for the current event's width, use it
            if space.x_range.start > current_x + width {
                return current_x;
            }
            current_x = current_x.max(space.x_range.last);
        }
        current_x
    }
}

pub(crate) struct OccupiedSpace {
    y_range: core::range::RangeInclusive<u16>,
    x_range: core::range::RangeInclusive<u16>,
}

pub(crate) fn draw_event<D>(
    display: &mut D,
    start: &jiff::Zoned,
    end: &jiff::Zoned,
    text: &str,
    start_display_hour: u8,
    today: &jiff::civil::Date,
    spaces: &mut OccupiedSpaces,
) where
    D: DrawTarget<Color = EpdColor> + OriginDimensions,
    D::Error: core::fmt::Debug,
{
    let start_days_diff = start.date().since(*today).unwrap().get_days();
    let start_min = start_days_diff * 24 * 60 + start.hour() as i32 * 60 + start.minute() as i32;

    let end_days_diff = end.date().since(*today).unwrap().get_days();
    let mut end_min = end_days_diff * 24 * 60 + end.hour() as i32 * 60 + end.minute() as i32;

    let display_start_mins = start_display_hour as i32 * 60;
    let display_end_mins = (start_display_hour as i32 + get_display_hours() as i32) * 60;

    if end_min <= display_start_mins || start_min >= display_end_mins {
        #[cfg(feature = "defmt")]
        crate::defmt::warn!(
            "Event '{}' is out of display bounds ({}-{}), skipping",
            text,
            crate::defmt::Debug2Format(&start),
            crate::defmt::Debug2Format(&end)
        );
        return;
    }

    let y = calculate_start_height(
        (start_min - display_start_mins).max(0) as u16,
        start_display_hour,
    );

    let is_past_midnight = end_min >= 1440;

    if DRAW_NEW_DAY == true && is_past_midnight {
        end_min += 60;
    }

    let mut end_y = calculate_end_height(
        (end_min - display_start_mins).max(0) as u16,
        start_display_hour,
    )
    .clamp(
        0,
        calculate_end_height(get_display_hours() as u16 * 60, start_display_hour),
    );

    let available_height = end_y.saturating_sub(y);

    const EVENT_LINE_HEIGHT: u32 = EVENT_FONT.character_size.height;
    const TIME_LINE_HEIGHT: u32 = MINI_FONT.character_size.height;
    const TITLE_WITH_TIME_HEIGHT: u32 = EVENT_LINE_HEIGHT + TIME_LINE_HEIGHT;

    let oneline = available_height < TITLE_WITH_TIME_HEIGHT;

    if available_height < TIME_LINE_HEIGHT {
        end_y = y + TIME_LINE_HEIGHT;
    }

    let mut owned_str: heapless::String<32> = heapless::String::new();

    {
        let max_multiline_cnt =
            available_height.saturating_sub(TIME_LINE_HEIGHT) / EVENT_LINE_HEIGHT;
        let mut lines = 0;
        let mut counted = 0;
        let mut truncated = false;
        for c in text.chars() {
            let char_len = c.len_utf8();
            // Check if we have room: 3 bytes reserved for "..."
            if owned_str.len() + char_len + 3 > owned_str.capacity() {
                truncated = true;
                break;
            }
            if !oneline && counted > 4 && c.eq(&' ') {
                if lines + 1 >= max_multiline_cnt {
                    truncated = true;
                    break;
                }
                owned_str.push('\n').ok();
                counted = 0;
                lines += 1;
            } else {
                owned_str.push(c).ok();
                counted += 1;
            }
        }
        if truncated || text.chars().count() > owned_str.capacity() - 3 {
            owned_str.push_str("...").unwrap();
        }
    };

    let time_str: heapless::String<15> =
        hformat!("{}-{}", start.strftime("%H:%M"), end.strftime("%H:%M")).unwrap();

    let estimated_width = if oneline {
        calculate_text_width(time_str.len() as u16, MINI_FONT)
            + calculate_text_width(text.len() as u16, EVENT_FONT)
            + 5
    } else {
        let max_len = owned_str.lines().map(|line| line.len()).max().unwrap_or(0);

        calculate_text_width(time_str.len() as u16, MINI_FONT)
            .max(calculate_text_width(max_len as u16, EVENT_FONT))
            + 5
    };

    let next_free_slot = spaces.get_next_free_slot(
        RangeInclusive::from(y as u16..=end_y as u16),
        estimated_width,
    );

    let x: i32 = START_POS + next_free_slot as i32;

    let time_text = Text::with_baseline(
        &time_str,
        Point::new(x, y as i32),
        MINI_CHARACTER_STYLE,
        embedded_graphics::text::Baseline::Top,
    );

    let title_point = if oneline {
        let time_width = time_text.bounding_box().size.width as i32;
        Point::new(x + time_width + 3, y as i32)
    } else {
        Point::new(x, y as i32 + MINI_FONT.character_size.height as i32)
    };

    let title_text = Text::with_baseline(
        &owned_str,
        title_point,
        CHARACTER_STYLE,
        embedded_graphics::text::Baseline::Top,
    );

    let time_bb = time_text.bounding_box();
    let title_bb = title_text.bounding_box();

    let min_x = time_bb.top_left.x.min(title_bb.top_left.x);
    let max_x = (time_bb.top_left.x + time_bb.size.width as i32)
        .max(title_bb.top_left.x + title_bb.size.width as i32);
    let max_y = (time_bb.top_left.y + time_bb.size.height as i32)
        .max(title_bb.top_left.y + title_bb.size.height as i32)
        .max(end_y as i32);

    let mut ebb =
        Rectangle::with_corners(Point::new(min_x, y as i32), Point::new(max_x as i32, max_y));

    extend_rectangle(&mut ebb);

    spaces.add_space(
        RangeInclusive::from(ebb.top_left.y as u16..=ebb.bottom_right().unwrap().y as u16),
        RangeInclusive::from(
            (ebb.top_left.x - START_POS).max(0) as u16
                ..=(ebb.bottom_right().unwrap().x + 2 - START_POS) as u16,
        ),
    );

    ebb.into_styled(OVERWRITE_STYLE).draw(display).unwrap();

    time_text.draw(display).unwrap();
    title_text.draw(display).unwrap();
}

pub const fn extend_rectangle(rec: &mut Rectangle) {
    rec.size.width += 3;
    rec.top_left.x -= 2;
}

#[allow(dead_code)]
pub fn draw_days<D>(display: &mut D, current_day: jiff::civil::Weekday, count: u8)
where
    D: DrawTarget<Color = EpdColor> + OriginDimensions,
    D::Error: core::fmt::Debug,
{
    let starting_x = START_POS;
    let y = display.bounding_box().size.height - EVENT_FONT.character_size.height;
    let mut x_offset = 0;
    // max 7 days supported
    let mut days: heapless::Vec<jiff::civil::Weekday, 7> = heapless::Vec::new();
    for i in 0..count.clamp(1, 7) {
        days.push(current_day.wrapping_add(i as i64)).unwrap();
    }
    for day in days {
        let day_text = match day {
            jiff::civil::Weekday::Monday => "Monday",
            jiff::civil::Weekday::Tuesday => "Tuesday",
            jiff::civil::Weekday::Wednesday => "Wednesday",
            jiff::civil::Weekday::Thursday => "Thursday",
            jiff::civil::Weekday::Friday => "Friday",
            jiff::civil::Weekday::Saturday => "Saturday",
            jiff::civil::Weekday::Sunday => "Sunday",
        };
        x_offset += day_text.chars().count() as i32 * EVENT_FONT.character_size.width as i32 + 15;
        let pos = Point::new(starting_x + x_offset, y as i32);
        #[cfg(feature = "defmt")]
        info!(
            "Drawing day '{}' at position {:?}",
            day_text,
            crate::defmt::Debug2Format(&pos)
        );

        Text::with_baseline(
            day_text,
            pos,
            CHARACTER_STYLE,
            embedded_graphics::text::Baseline::Top,
        )
        .draw(display)
        .unwrap();
    }
}

pub(crate) fn draw_config<D>(display: &mut D, text: &str)
where
    D: DrawTarget<Color = EpdColor> + OriginDimensions,
    D::Error: core::fmt::Debug,
{
    let text_style = embedded_graphics::text::TextStyleBuilder::new()
        .alignment(embedded_graphics::text::Alignment::Center)
        .baseline(embedded_graphics::text::Baseline::Middle)
        .build();

    Text::with_text_style(
        text,
        display.bounding_box().center(),
        CHARACTER_STYLE,
        text_style,
    )
    .draw(display)
    .unwrap();
}

#[cfg(not(target_arch = "xtensa"))]
pub use not_xtensa::*;
#[cfg(target_arch = "xtensa")]
pub use xtensa::*;

#[cfg(target_arch = "xtensa")]
pub mod xtensa {
    use alloc::string::ToString;

    use display_interface::AsyncWriteOnlyDataCommand;
    use embedded_hal::digital::OutputPin as EhalOutputPin;
    use embedded_hal_async::{delay::DelayNs, digital::Wait};
    use esp_hal::rtc_cntl::Rtc;
    use weact_studio_epd::WeActStudio420BlackWhiteDriver;
    pub use weact_studio_epd::graphics::Display420BlackWhite;

    use super::draw_event;
    use crate::{display::BACKGROUND_COLOR, hardware};

    pub(crate) async fn write_to_screen<DI, BSY, RST, DELAY>(
        display: &mut Display420BlackWhite,
        driver: &mut WeActStudio420BlackWhiteDriver<DI, BSY, RST, DELAY>,
        events: &mut [vcal_parser::vevent::VEventData],
        rtc: &mut Rtc<'_>,
    ) where
        DI: AsyncWriteOnlyDataCommand,
        BSY: embedded_hal::digital::InputPin + Wait,
        RST: EhalOutputPin,
        DELAY: DelayNs,
    {
        let time = hardware::get_time(rtc);
        let mut start_display_hour = time.hour() as u8;

        if !super::limit_to_today() {
            start_display_hour = start_display_hour.clamp(0, 24 - super::get_display_hours());
        }

        display.clear(BACKGROUND_COLOR);

        crate::display::draw_base_calendar(display, start_display_hour, &time);
        crate::display::draw_time_row_header(display, start_display_hour);
        let tz = jiff::tz::TimeZone::fixed(jiff::tz::offset(2));
        let mut spaces = super::OccupiedSpaces::new();

        events.sort();

        for event in events
            .iter()
            .filter(|f| f.dtstart.is_some() && f.dtend.is_some())
        {
            let start_dt = event.dtstart.unwrap().to_zoned(tz.clone());
            let end_dt = event.dtend.unwrap().to_zoned(tz.clone());
            draw_event(
                display,
                &start_dt,
                &end_dt,
                &event.summary.clone().unwrap_or("No summary".to_string()),
                start_display_hour,
                &time.date(),
                &mut spaces,
            );
        }
        #[cfg(debug_assertions)]
        crate::display::add_footer_info(display);

        if super::limit_to_today() {
            crate::display::draw_time_ticker(display, &time, start_display_hour);
        }
        crate::display::draw_sync_time(display, &time);
        driver.full_update(display).await.unwrap();

        crate::wifi::wait_until_wifi_stop().await;

        crate::hardware::go_to_deep_sleep(rtc);
    }
}

#[cfg(not(target_arch = "xtensa"))]
pub mod not_xtensa {
    use super::*;

    pub(crate) async fn write_to_screen() {
        #[cfg(feature = "defmt")]
        crate::defmt::info!("Mock writing to screen");
    }
}

pub fn date_to_mins(dt: &jiff::Zoned) -> u16 {
    dt.hour() as u16 * 60 + dt.minute() as u16
}
