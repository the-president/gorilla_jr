use std::io::{Write,stdout, stdin};
use std::error::Error;

use termion::event::{Event,Key};
use termion::{clear,color,cursor};
use termion::color::*;


use midir::{MidiInput, MidiOutput, Ignore};

#[derive(Copy,Clone,PartialEq,Eq)]
enum Col
{
	Input,
	Output
}

impl Col
{
	pub fn toggle(self) -> Col
	{
		match self
		{
			Col::Input => Col::Output,
			Col::Output => Col::Input
		}
	}
}

pub struct MidiSelectScreen
{
	count:u8,
	current_row :i16,
	current_col : Col,
	max_row : u16
}

const COL_WIDTH : u16 = 40;
const DIVIDER: &str = "================================================================================";

impl MidiSelectScreen
{
	pub fn new(input: &MidiInput, output:&MidiOutput) -> MidiSelectScreen
	{
		let in_max = input.port_count() as u16;
		let out_max = output.port_count() as u16;

		MidiSelectScreen{count:0,current_row:0,current_col: Col::Input,max_row: std::cmp::max(in_max,out_max) - 1}
	}

	pub fn draw(&self,input: &MidiInput, output:&MidiOutput) -> Result<(), Box<dyn Error>>
	{
		write!(stdout(),"{}",clear::All)?;
		write!(stdout(),"{}{}INPUT",cursor::Goto(18,1),Fg(Green))?;
		write!(stdout(),"{}{}OUTPUT",cursor::Goto(COL_WIDTH+17,1),Fg(Green))?;
		write!(stdout(),"{}{}{}",cursor::Goto(1,2),DIVIDER, Fg(Rgb(0xFF,0xFF,0xFF)))?;

		let in_line : u16 = 3;

		for i in 0..input.port_count() 
		{
			let current_y =  in_line + i as u16;
			if (i as u16) == (self.current_row as u16) && self.current_col == Col::Input
			{
				write!(stdout(),"{}{}        {}{}{}",cursor::Goto(4,current_y),i,Bg(Blue),input.port_name(i)?,Bg(Reset))?;
			}
			else 
			{
				write!(stdout(),"{}{}        {}",cursor::Goto(4,current_y ),i,input.port_name(i)?)?;
			}
		}

		let out_line : u16 = 3;

		for i in 0..output.port_count() 
		{
			let current_y =  out_line + i as u16;
			if (i as u16) == (self.current_row as u16) && self.current_col == Col::Output
			{
				write!(stdout(),"{}|{}{}        {}{}{}",cursor::Goto(40,current_y),cursor::Goto(44,current_y),i,Bg(Blue),output.port_name(i)?,Bg(Reset))?;
			}
			else 
			{
				write!(stdout(),"{}|{}{}        {}",cursor::Goto(40,current_y),cursor::Goto(44,current_y ),i,output.port_name(i)?)?;
			}
		}

		write!(stdout(),"{}{}",Fg(Reset),Bg(Reset))?;

		write!(stdout(),"{}{}  {}  {}",cursor::Goto(1,self.max_row + 4),Fg(Red),self.count,Fg(Reset))?;
		stdout().flush()?;
		
		Ok(())
	}

	pub fn input(&mut self,e : &Event)
	{
		match e
		{
			Event::Key(Key::Down) =>
			{
				self.current_row = std::cmp::min(self.max_row as i16,self.current_row + 1);
			},

			Event::Key(Key::Up) =>
			{
				self.current_row = std::cmp::max(0,self.current_row - 1);
			},

			Event::Key(Key::Left) => 
			{
				self.current_col = self.current_col.toggle();
			}

			Event::Key(Key::Right) =>
			{
				self.current_col = self.current_col.toggle();
			}


			_ => ()
		}
	}

	pub fn tick(&mut self)
	{
		self.count = self.count + 1;
	}
}