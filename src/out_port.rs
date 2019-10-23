use std::result::Result;
use midir::{MidiOutputConnection,SendError};

const NOTE_ON_STATUS: u8 = 0x90;
const NOTE_OFF_STATUS: u8 = 0x80;

pub fn note_on(conn: &mut MidiOutputConnection, channel: u8, nn: u8, vel: u8) -> Result<(),SendError>
{
	conn.send(&[NOTE_ON_STATUS | channel,nn,vel])
}

pub fn note_off(conn: &mut MidiOutputConnection, channel: u8, nn: u8, vel: u8) -> Result<(),SendError>
{
	conn.send(&[NOTE_OFF_STATUS | channel,nn,vel])
}