// function names

pub const GET: &str = "_w.g"; // GET(Id) returns the value at memory slot Id
pub const DEL: &str = "_w.d"; // DEL(Id) removes the value at memory slot Id
pub const SET: &str = "_w.s"; // SET(Id, Value) sets the value at memory slot Id
pub const REP: &str = "_w.r"; // REP(Id, Value) sends the value back as id:json(value)
pub const ERR: &str =  "_w.e"; //ERR(Id,Value) propagates an error
pub const CATCH: &str = "_w.c"; //CATCH(Id) catches an error
pub const IMPORT: &str = "_w.x";//IMPORT: an object of imports
pub const REPLY: &str = "_w.rp"; //REPLY(Id,Value): reply to a rpc request
pub const ALLOC: &str = "_w.a"; //ALLOC(Value) returns a new JS-created Id from Value