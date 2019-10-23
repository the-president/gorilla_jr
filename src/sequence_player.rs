use midir::{MidiOutputConnection};
use crate::sequence::{PlayState,Seq};

pub struct Player
{
	//global playback info (for syncing seq starts)
	tick_accum : usize,
	ticks_per_step : usize,

	pub midi_map : [Seq;128],

	pub out_connections: Vec<(usize,MidiOutputConnection)>,
}

impl Player
{

	pub fn blank() -> Player
	{
			Player
			{
				tick_accum : 0,
				ticks_per_step : 6,
				out_connections: vec![],
				midi_map:[Seq::blank();128]
			}
	}

	pub fn tick(&mut self) -> bool
	{
		self.tick_accum = self.tick_accum + 1;

		let hit = if self.tick_accum == self.ticks_per_step
		{
			self.tick_accum = 0;
			true
		}
		else 
		{
			false
		};

		for i in 0..128
		{
			if let Some(seq) = self.midi_map.get_mut(i)
			{
				match (seq.is_blank(),seq.state,seq.port,hit)
				{
					(false,PlayState::Playing,num,_) => 
					{
						if let Some((_i,port)) = self.out_connections.get_mut(num)
						{
							seq.tick(port);	
						}
					},

					(false,PlayState::Starting,num,true) =>
					{
						if let Some((_i,port)) = self.out_connections.get_mut(num)
						{
							seq.start(port);	
							seq.state = PlayState::Playing;
						}
					},

					(_,_,_,_) => ()
				}
			}
		}

		hit
	}

	pub fn note_on(&mut self, nn : usize)
	{
		eprintln!("NOTE ON!");
		let mut seq = &mut self.midi_map[nn];

		if seq.hold
		{
			match seq.state
			{
				PlayState::Off =>{
					seq.state = PlayState::Starting;
				} 
				_ => 
				{
					seq.state =PlayState::Off;

					if let Some((_i,port)) = self.out_connections.get_mut(seq.port)
					{
						seq.stop(port);
					}	
				}
				
			}
		}
		else 
		{
			seq.state = match seq.state
			{
				PlayState::Off => PlayState::Starting,
				_ => seq.state
			}	
		}
	}

	pub fn note_off(&mut self, nn : usize)
	{
		let mut seq = &mut self.midi_map[nn];
		
		if seq.hold
		{
			return
		}
		else 
		{
			seq.state = match seq.state
			{
				PlayState::Off => seq.state,
				_ => PlayState::Off
			};

			if let Some((_i,port)) = self.out_connections.get_mut(seq.port)
			{
				seq.stop(port);
			}	
		}
	}
}