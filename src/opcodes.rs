#[derive(Debug)]
pub enum Opcode {
	MOV,
	HLT,
	IN,
	OUT,
	JNZ,
	ADD,
	SUB,
	PUSH,
	POP,
	INT
}
