use crate::sequence::{self,Seq,Trigger};
use crate::note_lookup;
use crate::input_types::Input;
use crate::sequence_player::Player;
use crate::midi_msg::MidiMessage;


use termion::event::{Event,Key};
use termion::{clear,color,cursor};
use termion::color::*;

use std::io::{Write,stdout, stdin};

const BAR_SIZE:usize = 16;

fn draw_seq(mut screen : impl std::io::Write, x : u16,y : u16,seq : &Seq)
{
	write!(screen,"{}",cursor::Goto(x,y+1)).unwrap();

	for i in 0..seq.length
	{
		if i > 0 && i % BAR_SIZE == 0
		{
			match x
			{
				0 => write!(screen,"\r\n\n").unwrap(),
				n => write!(screen,"\r\n\n{}",cursor::Right(n - 1)).unwrap()
			};
		}

		let step = seq.steps[i];

		match (step.hold,i == seq.edit_step)
		{
			(true, true) =>write!(screen,"{}{}-X-{}{}{}",cursor::Up(1),Fg(Red),Fg(Reset),cursor::Down(1),cursor::Left(3)).unwrap(),
			(true,false) => write!(screen,"{}{} - {}{}{}",cursor::Up(1),Fg(Red),Fg(Reset),cursor::Down(1),cursor::Left(3)).unwrap(),
			(false,true) => write!(screen,"{}{}{}X{}{}{}",cursor::Right(1),cursor::Up(1),Fg(Red),Fg(Reset),cursor::Down(1),cursor::Left(2)).unwrap(),
			(_,_) => ()
		}
		
		match (step.trig,i==seq.position)
		{
			(Trigger::Off,false) => write!(screen,"{}---{} ",color::Fg(Green),color::Fg(Reset)),
			(Trigger::On(note,_),false) => write!(screen,"{}{}{} ",color::Fg(Red),note_lookup::note_str(note),color::Fg(Reset)),
			(Trigger::Off,true) => write!(screen,"{}---{} ",color::Fg(Magenta),color::Fg(Reset)),
			(Trigger::On(note,_),true) => write!(screen,"{}{}{} ",color::Fg(Magenta),note_lookup::note_str(note),color::Fg(Reset)),
		}.unwrap();
	}

	write!(screen,"{}{}\r\n\n{}",color::Fg(Reset),color::Bg(Reset),cursor::Right(x - 1)).unwrap();


	//ok now draw the ticks per step
	write!(screen,"ticks: {}    channel:{}    port:{}",seq.ticks_per_step,seq.channel,seq.port).unwrap();
}

enum Mode
{
	Edit,
	Play
}

struct EditState
{
	current_edit : usize
}

struct PlayState
{

}

pub struct Screen
{
	w : u16,
	h : u16,

	mode : Mode,
	edit_state: EditState,
	play_state : PlayState
}

impl Screen
{
	pub fn new() -> Screen
	{
		let (sw,sh) = (82,32);

		Screen
		{
			w: sw,
			h: sh,
			mode : Mode::Play,
			edit_state: EditState{current_edit:0},
			play_state : PlayState{}
		}
	}

	fn edit_midi_input(&mut self,player: &mut Player, evt:MidiMessage ) -> bool
	{
		let edit_seq = & mut player.midi_map[self.edit_state.current_edit];

		match evt
		{
			MidiMessage::NoteOn(_,nn,0)  =>
			{
				player.note_off(nn as usize);
				true
			},

			MidiMessage::NoteOn(_,nn,vel)  =>
			{
				edit_seq.set_step_note(nn,0x7F);
				edit_seq.edit_step_up_wrap();
				true
			},

			MidiMessage::NoteOff(c,nn,v) =>
			{
				player.note_off(nn as usize);
				true
			},

			_ => false
		}
	}

	fn edit_input(&mut self, player: &mut Player, evt:Input ) -> bool
	{
		let edit_seq = & mut player.midi_map[self.edit_state.current_edit];

		match evt
		{
			Input::Keyboard(Key::Left) =>
			{
				edit_seq.edit_step_down();
				true
			},

			Input::Keyboard(Key::Right) =>
			{
				edit_seq.edit_step_up_wrap();
				true
			}

			Input::Keyboard(Key::Up) =>
			{
				edit_seq.bar_down(BAR_SIZE);
				true
			},

			Input::Keyboard(Key::Down) =>
			{
				edit_seq.bar_up(BAR_SIZE);
				true
			}

			Input::Keyboard(Key::Char('x')) =>
			{
				edit_seq.set_step_off();
				edit_seq.edit_step_up_wrap();
				true
			}

			Input::Keyboard(Key::Char('h')) =>
			{
				edit_seq.toggle_step_hold();
				true
			}

			Input::Keyboard(Key::Char('-')) =>
			{
				edit_seq.drop_step();
				true
			}

			Input::Keyboard(Key::Char('+')) =>
			{
				edit_seq.add_step();
				true
			}

			Input::Keyboard(Key::Char('>')) =>
			{
				edit_seq.up_tick_rate();
				true
			}

			Input::Keyboard(Key::Char('<')) =>
			{
				edit_seq.down_tick_rate();
				true
			}

			Input::Keyboard(Key::Char('[')) =>
			{
				edit_seq.down_channel();
				true
			}

			Input::Keyboard(Key::Char(']')) =>
			{
				edit_seq.up_channel();
				true
			}

			Input::Keyboard(Key::Char(';')) =>
			{
				edit_seq.down_port();
				true
			}

			Input::Keyboard(Key::Char('\'')) =>
			{
				edit_seq.up_port(player.out_connections.len() - 1);
				true
			}

			Input::Midi(msg) =>
			{
				self.edit_midi_input(player,msg)
			}

			_ => false
		}
	}

	fn play_input(&mut self, player:&mut Player, evt:Input ) -> bool
	{
		match evt
		{
			Input::Midi(MidiMessage::NoteOn(c,nn,0)) =>
			{
				player.note_off(nn as usize);
				true
			},

			Input::Midi(MidiMessage::NoteOn(c,nn,v)) =>
			{
				player.note_on(nn as usize);
				self.edit_state.current_edit = nn as usize;
				true
			},

			Input::Midi(MidiMessage::NoteOff(c,nn,v)) =>
			{
				player.note_off(nn as usize);
				true
			},

			Input::Keyboard(Key::Char('h')) =>
			{
				player.midi_map[self.edit_state.current_edit].hold = !player.midi_map[self.edit_state.current_edit].hold;
				true
			}

			_ => false
		}
	}

	pub fn input(&mut self, player:&mut Player, evt:Input ) -> (bool,bool) //when to redraw, and when to quit
	{
		//universal stuff
		let (quit,rd1) = match evt
		{
			Input::Quit => (true,false),

			Input::Tick => (false,player.tick()),

			Input::Keyboard(Key::F(1)) =>
			{
				eprintln!("SETTING MODE TO PLAY");
				self.mode = Mode::Play;
				(false,true)
			},
			Input::Keyboard(Key::F(2)) =>
			{
				eprintln!("SETTING MODE TO EDIT");
				self.mode = Mode::Edit;
				(false,true)
			},
			_=>(false,false)
		};

		let rd2 = match self.mode
		{
			Mode::Edit => self.edit_input(player,evt),
			Mode::Play => self.play_input(player,evt)
		};

		return (rd1||rd2,quit)
	}

	fn draw_play_screen(&self,player:&Player)
	{
		write!(stdout(),"{}",cursor::Goto(1,3)).unwrap();

		let playing_seqs = player.midi_map.iter()
		.enumerate()
		.filter( |(i,s)| 
		{
			s.state == sequence::PlayState::Starting || s.state == sequence::PlayState::Playing
		})
		.take(16);

		for (i,seq) in playing_seqs
		{
			if i == self.edit_state.current_edit
			{
				write!(stdout(),"{}",termion::style::Underline).unwrap();
			}

			match (seq.state,seq.hold)
			{
				(sequence::PlayState::Playing,true) => write!(stdout(),"{}{}{}\n\r",Bg(Magenta),note_lookup::note_str(i as u8),Bg(Reset)).unwrap(),
				(sequence::PlayState::Playing,false) => write!(stdout(),"{}\n\r",note_lookup::note_str(i as u8)).unwrap(),
				(sequence::PlayState::Starting,_) => write!(stdout(),"{}{}{}\n\r",Bg(Cyan),note_lookup::note_str(i as u8),Bg(Reset)).unwrap(),
				_=>()
			}

			write!(stdout(),"{}",termion::style::NoUnderline).unwrap();
		}
	}

	fn draw_top_panel(&self)
	{
		write!(stdout(),"{}{}{}{} f1:play   f2: edit\n\r",cursor::Goto(1,1),clear::CurrentLine,Fg(Reset),Bg(Reset)).unwrap();
		
		for _i in 0 .. self.w
		{
			write!(stdout(),"=").unwrap();
		}
	}

	fn clear_main_screen(&self)
	{
		write!(stdout(),"{}{}",cursor::Goto(1,3),clear::AfterCursor).unwrap();
	}

	pub fn draw(&self,player:&Player,)
	{
		self.draw_top_panel();
		self.clear_main_screen();

		match self.mode
		{
			Mode::Edit => 
			{
				draw_seq(stdout(),9,3,&player.midi_map[self.edit_state.current_edit])
			},

			Mode::Play =>
			{
				self.draw_play_screen(player)
			}
		}
	}
}
