#[derive(Debug)]
pub enum MidiMessage
{
	//3 byte channel messages
	NoteOff(u8,u8,u8),
	NoteOn(u8,u8,u8),
	PolyPressure(u8,u8,u8),
	ControlChange(u8,u8,u8),
	PitchBend(u8,u8,u8),

	//2 byte messages channel messages
	AfterTouch(u8,u8),
	ProgramChange(u8,u8),

	//reserved controller messages
	AllSoundOff(u8),
	ResetControllers(u8,u8),
	LocalControl(u8,bool),
	AllNotesOff(u8),
	OmniModeOff(u8),
	OmniModeOn(u8),
	MonoModeOn(u8,u8),
	PolyModeOn(u8),

	//systemCommons
	MTCframe(u8,u8),
	SongPosition(u8,u8),
	SongSelect(u8),
	TuneRequest,
	//sys real times
	Tick,
	Start,
	Continue,
	Stop,
	ActiveSense,
	Reset,

	SysEx(Vec<u8>),

	None //this basically means it couldn't be parsed
}

fn parse_cc(status:u8 ,data1: u8,data2: u8) -> MidiMessage
{
	let channel = status &0x0F;

	match data1
	{
		0 ..= 119 => MidiMessage::ControlChange(channel,data1,data2),
		120 => MidiMessage::AllSoundOff(channel),
		121 => MidiMessage::ResetControllers(channel,data2),
		122 => MidiMessage::LocalControl(channel,data2 == 127),
		123 => MidiMessage::AllNotesOff(channel),
		124 => MidiMessage::OmniModeOff(channel),
		125 => MidiMessage::OmniModeOn(channel),
		126 => MidiMessage::MonoModeOn(channel,data2),
		127 => MidiMessage::PolyModeOn(channel),
		_ => MidiMessage::None
	}
}

pub fn parse(message : &[u8] ) ->  MidiMessage
{
	if message.len() == 0
	{  
		MidiMessage::None
	}
	else 
	{
		match message[0]
		{
			0b10000000 ..= 0b10001111 => MidiMessage::NoteOff(message[0]&0x0F,message[1],message[2]),
			0b10010000 ..= 0b10011111 => MidiMessage::NoteOn(message[0]&0x0F,message[1],message[2]),
			0b10100000 ..= 0b10101111 => MidiMessage::PolyPressure(message[0]&0x0F,message[1],message[2]),
			0b11010000 ..= 0b11011111 => MidiMessage::AfterTouch(message[0]&0x0F,message[1]),
			0b11100000 ..= 0b11101111 => MidiMessage::PitchBend(message[0]&0x0F,message[1],message[2]),
			0b10110000 ..= 0b10111111 => parse_cc(message[0],message[1],message[2]),
			0b11000000 ..= 0b11001111 => MidiMessage::ProgramChange(message[0]&0x0F,message[1]),
			0b11110001 => 
			{
				let msg_type = (message[1] & 0b01110000) >> 4;
				let vals = message[1] & 0x0F;
				MidiMessage::MTCframe(msg_type,vals)
			},
			0b11110010	=> MidiMessage::SongPosition(message[1],message[2]),
			0b11110011 => MidiMessage::SongSelect(message[1]),
			0b11110110 => MidiMessage::TuneRequest,

			0b11111000 => MidiMessage::Tick,
			0b11111010 =>	MidiMessage::Start,
			0b11111011 => MidiMessage::Continue,
			0b11111100 => MidiMessage::Stop,
			0b11111110 => MidiMessage::ActiveSense,
			0b11111111 => MidiMessage::Reset,
			
			0b011110000 => MidiMessage::SysEx((&message[1..message.len()-1]).to_vec()),

			_ => MidiMessage::None
		}
	}
}