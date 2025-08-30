use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sql_cli::ui::key_handling::{ChordResult, KeyChordHandler};

fn main() {
    println!("Testing chord handler...");

    let mut handler = KeyChordHandler::new();

    // Test yv chord
    println!("\nTesting 'yv' chord:");

    let y = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty());
    println!("Sending 'y': {:?}", y);
    let result = handler.process_key(y);
    println!("Result: {:?}", result);

    let v = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::empty());
    println!("Sending 'v': {:?}", v);
    let result = handler.process_key(v);
    println!("Result: {:?}", result);

    println!("\n---Reset handler---\n");
    handler = KeyChordHandler::new();

    println!("Testing 'yy' chord:");
    let y1 = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty());
    println!("Sending 'y': {:?}", y1);
    let result = handler.process_key(y1);
    println!("Result: {:?}", result);

    let y2 = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty());
    println!("Sending 'y': {:?}", y2);
    let result = handler.process_key(y2);
    println!("Result: {:?}", result);
}
