cbindgen ../../runtime-c/ -o cuentitos.hpp
g++ -std=c++17 cuentitos.cpp "../../target/debug/libcuentitos_runtime_c.a" -o cuentitos 
