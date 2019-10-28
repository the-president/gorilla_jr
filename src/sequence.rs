use crate::out_port;
use midir::MidiOutputConnection;

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum PlayState
{
	Off,
	Starting,
	Playing,
}

#[derive(Debug,Copy,Clone)]
pub enum Trigger
{
	Off,
	On (u8,u8)
}

#[derive(Debug,Copy,Clone)]
pub struct Step
{
	pub trig:Trigger,
	pub hold:bool
}

const DEFAULT_STEP : Step = Step{trig:Trigger::Off,hold:false};

//for now sequences are fixed max,
const SEQ_MAX:usize  = 64;

//gonna try this
#[derive(Copy,Clone)]
pub struct Seq //this also contains playback and edit info
{
	pub note_on: Option<u8>,
	pub steps: [Step;SEQ_MAX],
	pub length: usize,
	pub ticks_per_step:usize,

	//play info
	tick_accum: usize,
	pub position:usize,
	pub channel: u8,
	pub state : PlayState,
	pub hold : bool,
	pub port: usize,

	//edit info
	pub edit_step : usize	
}

impl Seq
{
	pub fn step_on(& mut self, step : usize, nn : u8)
	{
		let idx = std::cmp::min(step,SEQ_MAX);
		self.steps[idx].trig = Trigger::On(nn,127);
	}

	pub fn step_off(& mut self, step : usize)
	{
		let idx = std::cmp::min(step,SEQ_MAX);
		self.steps[idx].trig = Trigger::Off;
	}

	pub fn blank() -> Seq
	{
		Seq
		{
			steps: [DEFAULT_STEP;SEQ_MAX],
			length:0,
			ticks_per_step:6,
			tick_accum:0,
			position:0,
			note_on:None,
			state: PlayState::Off,
			hold: false,
			port: 0,
			channel:0,
			edit_step:0
		}
	}

	pub fn is_blank(&self) -> bool
	{
		self.length == 0
	}

	fn turn_off_playing_note(&mut self,con : &mut MidiOutputConnection)
	{
		match self.note_on
		{
			Some(nn) => 
			{
				out_port::note_off(con,self.channel,nn,127).unwrap();
				self.note_on = None;
			}
			_ => ()
		}
	}

	fn note_trigger(&mut self,con : &mut MidiOutputConnection) -> bool
	{
		match self.steps[self.position].trig
		{
			Trigger::On(nn,vel) => 
			{
				eprintln!("DOIN A NOTE ON");
				out_port::note_on(con,self.channel,nn,vel).unwrap();
				self.note_on = Some(nn);
				true
			}
			_=> false
		}
	}

	pub fn tick(& mut  self, con : &mut MidiOutputConnection)
	{
		self.tick_accum = self.tick_accum + 1;

		if self.tick_accum < self.ticks_per_step { return }
			
		self.position = (self.position + 1) % self.length;
		self.tick_accum = 0;

		let hold = self.steps[self.position].hold;

		if hold
		{
			if self.note_trigger(con) 
			{ 
				self.turn_off_playing_note(con) 
			};
		}
		else 
		{
			self.turn_off_playing_note(con);
			self.note_trigger(con);
		};
	}

	pub fn start(& mut  self, con : &mut MidiOutputConnection)
	{		
		self.position = 0;
		self.tick_accum = 0;

		//now check for a note on
		match self.steps[self.position].trig
		{
			Trigger::On(nn,vel) => 
			{
				out_port::note_on(con,self.channel,nn,vel).unwrap();
				self.note_on = Some(nn)
			}
			_=> ()
		}
	}

	pub fn stop(& mut  self, con : &mut MidiOutputConnection)
	{		
		self.position = 0;
		self.tick_accum = 0;

		//first check for a note off
		match self.note_on
		{
			Some(nn) => 
			{
				out_port::note_off(con,self.channel,nn,127).unwrap();
				self.note_on = None;
			}
			_ => ()
		}
	}

	pub fn edit_step_down(&mut self)
	{
		self.edit_step = if self.edit_step == 0
		{
			0
		}
		else 
		{
			self.edit_step - 1
		}
	}

	pub fn edit_step_up(&mut self)
	{
		self.edit_step = if self.edit_step == self.length - 1
		{
			self.length -1
		}
		else 
		{
			self.edit_step + 1
		}
	}

	pub fn edit_step_down_wrap(&mut self)
	{
		self.edit_step = if self.edit_step == 0
		{
			self.length - 1
		}
		else 
		{
			self.edit_step - 1
		}
	}

	pub fn edit_step_up_wrap(&mut self)
	{
		if self.length == 0
		{
			return
		}

		self.edit_step = if self.edit_step == self.length - 1
		{
			0
		}
		else 
		{
			self.edit_step + 1
		}
	}

	pub fn set_step_note(&mut self, nn:u8, vel:u8)
	{
		let mut step = & mut self.steps[self.edit_step];
		step.trig = Trigger::On(nn,vel);
	}

	pub fn set_step_off(&mut self)
	{
		let mut step = & mut self.steps[self.edit_step];
		step.trig = Trigger::Off;
	}

	pub fn toggle_step_hold(&mut self)
	{
		let mut step = & mut self.steps[self.edit_step];
		step.hold = !step.hold;
	}

	pub fn drop_step(&mut self)
	{
		self.length = match self.length
		{
			0 => 0,
			n => n-1
		}
	}

	pub fn add_step(&mut self)
	{
		self.length = match self.length
		{
			SEQ_MAX => SEQ_MAX,
			n => n+1
		}
	}

	pub fn bar_up(&mut self,bar_size: usize)
	{
		let new_step = bar_size + self.edit_step;

		self.edit_step = if new_step < self.length
		{
			new_step
		}
		else 
		{
			self.edit_step
		}
	}

	pub fn bar_down(&mut self,bar_size: usize)
	{
		self.edit_step = if self.edit_step >= bar_size
		{
			self.edit_step - bar_size
		}
		else 
		{
			self.edit_step
		}	
	}

	pub fn up_tick_rate(&mut self)
	{
		self.ticks_per_step = if self.ticks_per_step == 192
		{
			192
		}
		else 
		{
			self.ticks_per_step + 1
		}
	}

	pub fn down_tick_rate(&mut self)
	{
		self.ticks_per_step = if self.ticks_per_step == 1
		{
			1
		}
		else 
		{
			self.ticks_per_step - 1
		}
	}

	pub fn up_channel(&mut self)
	{
		self.channel = if self.channel == 0x0F
		{
			0x0F
		}
		else 
		{
			self.channel + 1
		}
	}

	pub fn down_channel(&mut self)
	{
		self.channel = if self.channel == 0
		{
			0
		}
		else 
		{
			self.channel - 1
		}
	}

	pub fn up_port(&mut self,max_port:usize)
	{
		self.port = if self.port as usize == max_port
		{
			max_port
		}
		else 
		{
			self.port + 1
		}
	}

	pub fn down_port(&mut self)
	{
		self.port = if self.port == 0
		{
			0
		}
		else 
		{
			self.port - 1
		}
	}
}