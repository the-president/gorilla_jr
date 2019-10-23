
use crate::midi_msg::MidiMessage;
use termion::event::{Event,Key};
use std::sync::mpsc::Sender;
use std::io::Stdin;

pub enum Input
{
	Keyboard(Key),
	Midi(MidiMessage),
	Tick,
	Quit
}

pub fn key_thread(tx : &Sender<Input>, evt : Event) -> bool
{
	match evt
	{
		Event::Key(Key::Char('Q')) =>
		{
			tx.send(Input::Quit).unwrap();
			true
		} 

		Event::Unsupported(_) => false,

		Event::Key(k) =>
		{
			tx.send(Input::Keyboard(k)).unwrap();
			false
		}

		_ => false
	}
}