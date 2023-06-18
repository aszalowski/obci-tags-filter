use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::io::Cursor;
use std::str;

fn main() {
    let mut reader = Reader::from_file("test_data/p6/N400_ab6c_PW2.tag").unwrap();
    reader.trim_text(true);
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), 32, 2);
    let mut tag_buf = Vec::new();
    let mut inner_tag_buf = Vec::new();
    let mut skip_tag = false;
    let mut writer_buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut tag_buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"tag" => {
                writer_buffer.push(Event::Start(e.into_owned()));
                loop {
                    match reader.read_event_into(&mut inner_tag_buf) {
                        Ok(Event::Start(inner_tag)) if inner_tag.name().as_ref() == b"nazw_obr" => {
                            writer_buffer.push(Event::Start(inner_tag.into_owned()));
                            let mut stimulus_name_buf = Vec::new();
                            match reader.read_event_into(&mut stimulus_name_buf) {
                                Ok(Event::Text(stimulus_name)) => {
                                    if str::from_utf8(stimulus_name.as_ref())
                                        .unwrap()
                                        .contains("lalka")
                                    {
                                        skip_tag = true;
                                    } else {
                                        writer_buffer.push(Event::Text(stimulus_name.into_owned()));
                                    }
                                }
                                _ => panic!("Expected text after tag."),
                            };
                        }
                        Ok(Event::End(e)) if e.name().as_ref() == b"tag" => {
                            if skip_tag {
                                writer_buffer.clear();
                                skip_tag = false
                            } else {
                                writer_buffer.push(Event::End(e.into_owned()));
                                for event in writer_buffer.iter() {
                                    assert!(writer.write_event(event).is_ok());
                                }
                                writer_buffer.clear()
                            }
                            break;
                        }
                        Ok(Event::Eof) => break,
                        Ok(e) => writer_buffer.push(e.into_owned()),
                        Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                    }
                }
            }
            Ok(Event::Eof) => break,
            Ok(e) => assert!(writer.write_event(e).is_ok()),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }

    let result = writer.into_inner().into_inner();
    println!("{}", str::from_utf8(result.as_slice()).unwrap())
}
