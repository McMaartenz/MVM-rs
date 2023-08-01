#[derive(Debug)]
pub enum Opcode {
	MOV, //x
	HLT, //x
	IN,  //x
	OUT, //x
	JNZ, //x
	ADD, //x
	SUB, //x
	PUSH,
	POP,
	INT
}
