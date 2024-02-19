md build\debug\windows\ 
md build\release\windows\

cargo build 
cargo build --release

cp ..\target\debug\cuentitos_godot.dll build\debug\windows\
cp ..\target\release\cuentitos_godot.dll build\release\windows\

