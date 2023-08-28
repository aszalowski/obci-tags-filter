use quick_xml::events::Event;
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use quick_xml::Error;
use std::io::{BufRead, Write};

const MAIN_TAGS_NAME: &[u8; 4] = b"tags";
const MAIN_TAG_NAME: &[u8; 3] = b"tag";
const IMG_TAG_NAME: &[u8; 8] = b"nazw_obr";
const SOUND_TAG_NAME: &[u8; 7] = b"nazw_dz";

const INDENT_CHAR: u8 = 9; // TAB
const INDENT_SIZE: usize = 1;

fn is_sound_or_img_tag(name: &[u8]) -> bool {
    name == IMG_TAG_NAME || name == SOUND_TAG_NAME
}

fn write_and_skip_tags<R, W>(
    reader: &mut Reader<R>,
    writer: &mut Writer<W>,
    exclude_words: &[String],
) -> Result<(), Error>
where
    R: BufRead,
    W: Write,
{
    let mut event_buf = Vec::new();
    let mut writer_buf = Vec::new();

    let mut event_count = 0;
    let mut skip_tags = false;

    loop {
        match reader.read_event_into(&mut event_buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == MAIN_TAG_NAME => {
                event_count += 1;
                writer_buf.push(Event::Start(e.into_owned()));
            }
            Ok(Event::Start(e)) if is_sound_or_img_tag(e.name().as_ref()) && !skip_tags => {
                writer_buf.push(Event::Start(e.into_owned()));

                match reader.read_event_into(&mut event_buf) {
                    Ok(Event::Text(e)) => {
                        skip_tags = exclude_words.iter().any(|word| {
                            e.unescape()
                                .expect("cannot unescape")
                                .as_ref()
                                .contains(word)
                        });
                        if !skip_tags {
                            writer_buf.push(Event::Text(e.into_owned()));
                        }
                    }
                    _ => panic!("Expected text after tag."),
                };
            }
            Ok(Event::End(e)) if e.name().as_ref() == MAIN_TAG_NAME => {
                if event_count % 2 != 0 {
                    writer_buf.push(Event::End(e.into_owned()));
                } else {
                    if skip_tags {
                        writer_buf.clear();
                    } else {
                        for event in writer_buf.iter() {
                            writer.write_event(event)?;
                        }
                        writer.write_event(Event::End(e))?;
                        writer_buf.clear();
                    }
                    skip_tags = false;
                }
            }
            Ok(Event::End(e)) if e.name().as_ref() == MAIN_TAGS_NAME => {
                writer.write_event(Event::End(e))?;
                break;
            }
            Ok(Event::Eof) => panic!("Unexpected EOF."),
            Ok(e) if !skip_tags => writer_buf.push(e.into_owned()),
            Ok(_) => {}
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }

    Ok(())
}

pub fn process_xml<R, W>(input: R, output: &mut W, exclude_words: &[String])
where
    R: BufRead,
    W: Write,
{
    let mut reader = Reader::from_reader(input);
    let mut writer = Writer::new_with_indent(output, INDENT_CHAR, INDENT_SIZE);

    let mut writer_buf = Vec::new();

    loop {
        match reader.read_event_into(&mut writer_buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == MAIN_TAGS_NAME => {
                assert!(writer.write_event(Event::Start(e)).is_ok());
                write_and_skip_tags(&mut reader, &mut writer, exclude_words).unwrap();
            }
            Ok(Event::Eof) => break,
            Ok(e) => assert!(writer.write_event(e).is_ok()),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }
}
