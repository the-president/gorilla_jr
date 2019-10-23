extern crate serde;
extern crate serde_json;

use serde::{Serialize,Deserialize};

use crate::sequence_player;
use crate::sequence_player::Player;
use crate::sequence::{Trigger,Seq};

use std::fs::{File,OpenOptions};
use std::io::BufReader;
use std::io::prelude::*;
use std::error::Error;
use std::fmt;

use midir::{MidiInput, MidiOutput, Ignore};

#[derive(Serialize,Deserialize)]
struct NoteConfig
{
	nn:u8,
	vel:u8
}

#[derive(Serialize,Deserialize)]
struct StepConfig
{
	note:Option<NoteConfig>,
	hold:bool,
}

#[derive(Serialize,Deserialize)]
struct SeqConfig
{
	midi_map:usize,
	channel:u8,
	tick_rate:usize,
	port:Option<usize>,
	steps:Vec<StepConfig>,
	hold:bool
}

#[derive(Serialize,Deserialize,Copy,Clone)]
pub struct MidiInConfig
{
	pub port_num:usize,
	pub ticks:bool
}

#[derive(Serialize,Deserialize)]
pub struct Config
{
	pub in_ports:Vec<MidiInConfig>,
	pub out_ports:Vec<usize>,
	seqs:Vec<SeqConfig>,

	#[serde(default="def_bpm")]
	pub bpm:f64
}

//this has to be a function for some reason
fn def_bpm() -> f64
{
	120.0
}

//=============================================================================
// READING THE CONFIG
//=============================================================================
pub fn read_config(path:&str) -> Result<Config,Box<dyn Error>>
{
	let file = File::open(path)?;
  let reader = BufReader::new(file);

  let conf = serde_json::from_reader(reader)?;

  Ok(conf)
}

fn set_seq(i:usize, entry : &SeqConfig, player: &mut Player) -> Result<(),ConfError>
{
	if entry.midi_map > 127
	{
		return Err(ConfError::new(format!("sequence {} has out of range midi_map",i)))
	}

	if entry.channel > 15
	{
		return Err(ConfError::new(format!("sequence {} has out of range channel",i)))
	}

	if entry.steps.len() > 64
	{
		return Err(ConfError::new(format!("sequence {} has too long a sequence",i)))	
	}

	let seq = &mut player.midi_map[entry.midi_map];

	seq.channel = entry.channel;
	seq.ticks_per_step = entry.tick_rate;
	seq.length = entry.steps.len();
	seq.hold = entry.hold;


	for (stepnum,step) in entry.steps.iter().enumerate()
	{
		let target_step = &mut seq.steps[stepnum];

		target_step.trig = if let Some(NoteConfig{nn,vel}) = step.note
		{
			Trigger::On(nn,vel)
		}
		else 
		{
			Trigger::Off
		};

		target_step.hold = step.hold;
	}

	if let Some(pnum) = entry.port
	{
		seq.port = pnum;
	}

	Ok(())
}

pub fn apply_config(player: &mut Player, conf: & Config) -> Result<(),ConfError>
{
	for (i,entry) in conf.seqs.iter().enumerate()
	{
		set_seq(i,entry,player)?;
	}

	for (i,pnum) in conf.out_ports.iter().enumerate()
	{
		let midi_out = MidiOutput::new("WOWOOWOOOWOOW").unwrap();
		let con = midi_out.connect(*pnum,"some thing").unwrap();

		player.out_connections.push((i,con));
	}

	Ok(())
}

//=============================================================================
// SAVING THE CONFIG
//=============================================================================
fn seq_to_config(i:usize,seq: &Seq) -> Result<SeqConfig,ConfError>
{
	let mut s_conf = SeqConfig
	{
		midi_map:i,
		channel:seq.channel,
		tick_rate:seq.ticks_per_step,
		steps:vec![],
		hold:seq.hold,
		port:Some(seq.port)
	};

	for step in seq.steps[0..seq.length].iter()
	{
		let note_conf = match step.trig
		{
			Trigger::On(nn,vel) => StepConfig{note:Some(NoteConfig{nn:nn,vel:vel}),hold:step.hold},
			Trigger::Off => StepConfig{note:None,hold:step.hold}
		};

		s_conf.steps.push(note_conf);
	}

	Ok(s_conf)
}

fn to_config(player: &Player,bpm:f64,in_ports : &[MidiInConfig]) -> Result<Config,ConfError>
{
	let mut conf = Config
	{
		bpm:bpm,
		in_ports:vec![],
		out_ports:vec![],
		seqs:vec![]
	};

	for (i,_port) in player.out_connections.iter()
	{
		conf.out_ports.push(*i);
	}

	for (i,seq) in player.midi_map.iter().enumerate()
	{
		if !seq.is_blank()
		{
			let seq_conf = seq_to_config(i,seq)?;
			conf.seqs.push(seq_conf);
		}
	}

	conf.in_ports.extend_from_slice(in_ports);

	Ok(conf)
}

pub fn save_config(path:&str, player: &Player,bpm:f64,in_ports: &[MidiInConfig]) -> Result<(),Box<dyn Error>>
{
	let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;

	let conf = to_config(player,bpm,in_ports)?;

	let conf_str = serde_json::to_string(&conf)?;

	eprintln!("THE LAST BIT");
	file.write_all(conf_str.as_bytes())?;

	Ok(())
}

#[derive(Debug)]
pub struct ConfError
{
	details: String
}

impl ConfError 
{
	fn new(msg: String) -> ConfError 
	{
		ConfError{details: msg}
	}
}

impl fmt::Display for ConfError 
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result 
	{
		write!(f,"{}",self.details)
	}
}

impl Error for ConfError 
{
	fn description(&self) -> &str 
	{
		&self.details
	}
} 