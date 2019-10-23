#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate termion;
extern crate midir;
extern crate adi_clock;
extern crate serde;
extern crate serde_json;

mod sequence;
mod note_lookup;
mod midi_msg;
mod input_types;
mod out_port;
mod sequence_player;
mod screens;
mod config_file;

use config_file::MidiInConfig;

use screens::Screen;

use sequence::{Seq,Trigger};

use input_types::Input;
use midi_msg::MidiMessage;

use termion::input::{TermRead};
use termion::raw::IntoRawMode;
use termion::event::{Event,Key};
use termion::{clear,color,cursor};
use termion::color::*;

use midir::{MidiInput,MidiInputConnection, MidiOutput, Ignore};

use std::io::{Write,stdout, stdin};
use std::error::Error;
use std::thread;
use std::sync::{atomic::{Ordering,AtomicBool},mpsc::{self,TryRecvError},Arc};
use std::time::Duration;

fn setup_seqs(file_path : Option<&str> ) -> Result<(sequence_player::Player,f64,Vec<MidiInConfig>), Box<dyn Error>> 
{
	let mut playo = sequence_player::Player::blank();

	let real_path = file_path.unwrap_or("./conf.json");
	let conf = config_file::read_config(real_path)?;

	config_file::apply_config(& mut playo,&conf)?;

	Ok((playo,conf.bpm,conf.in_ports))
}

fn save_conf(file_path : Option<&str>,player: &sequence_player::Player,bpm : f64, in_ports: &[config_file::MidiInConfig]) -> Result<(),Box<dyn Error>>
{
	let real_path = file_path.unwrap_or("./conf.json");

	config_file::save_config(real_path,player,bpm,in_ports)
}

fn real_main() -> Result<(), Box<dyn Error>>
{
	let (mut playo,bpm,in_ports) = setup_seqs(None)?;

	let _stdout = stdout().into_raw_mode().unwrap();

	let hangup = Arc::new(AtomicBool::new(false));
	let timer_hangup = hangup.clone();
	
	let (tx,rx) = mpsc::channel();

	let key_tx = tx.clone();

	let mut midi_in_connections : Vec<MidiInputConnection<()>> = vec![];
	let mut external_ticks = false;

	for midi_conf in in_ports.iter()
	{
		let mut midi_in = MidiInput::new("midi input").unwrap();
		let midi_tx = tx.clone();

		let in_con = if midi_conf.ticks
		{
			external_ticks = true;
			midi_in.ignore(Ignore::Sysex | Ignore::ActiveSense);

			midi_in.connect(midi_conf.port_num,"a midi port", move |_stamp,message,_|
	  	{
	  		let parsed = midi_msg::parse(message);

	  		let out_msg = match parsed
	  		{
	  			MidiMessage::Tick=> Input::Tick,
	  			_=>Input::Midi(parsed)
	  		};

	  		midi_tx.send(out_msg).unwrap();
	  	},()).unwrap()
		}
		else 
		{
			midi_in.ignore(Ignore::Sysex | Ignore::Time | Ignore::ActiveSense);
			
			midi_in.connect(midi_conf.port_num,"a midi port", move |_stamp,message,_|
	  	{
	  		midi_tx.send(Input::Midi(midi_msg::parse(message))).unwrap();
	  	},()).unwrap()
		};

  	



	  midi_in_connections.push(in_con);
	}

	let timer_thread = if !external_ticks
	{
		let spt = 1.0 / (24.0 * bpm / 60.0);
		Some(thread::spawn(move ||
		{
			let mut timer = adi_clock::Timer::new(spt as f32);

			while ! (timer_hangup.load(Ordering::Relaxed))
			{
				timer.wait();
				tx.send(Input::Tick).unwrap();
			}
		}))
	}
	else 
	{
		None	
	};

	let key_thread = thread::spawn(move ||
	{
		for k in stdin().events()
		{
			if input_types::key_thread(&key_tx,k.unwrap())
			{
				break;
			}
		}
	});

	let mut screen = Screen::new();

	print!("{}",cursor::Hide);
	screen.draw(&playo);
	stdout().flush()?;
	
	for event in rx.iter()
	{
		let (redraw,quit) = screen.input(& mut playo,event);

		if quit
		{ 
			break;
		}

		if redraw
		{
			screen.draw(&playo);
			stdout().flush()?;
		}
	}

	hangup.store(true,Ordering::Relaxed);
	
	if let Some(thrd) = timer_thread
	{
		thrd.join().unwrap();
	}

	key_thread.join().unwrap();

	//ok time to save the config

	save_conf(None,&playo,bpm,&in_ports[..])?;

	println!("{}{}{}",clear::All,cursor::Goto(1,1),cursor::Show);
	Ok(())
}

//this is so we can use the ?
fn main()
{
	match real_main()
	{
		Ok(_) => (),
		Err(err) => println!("It all went wrong: {}",err)
	};
}