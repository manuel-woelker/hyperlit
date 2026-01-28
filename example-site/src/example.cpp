// 📖 # This is a C++ example file demonstrating basic syntax and functionality

#include <iostream>
#include <string>

std::string greet(const std::string& name) {
    return "Hello, " + name + "!";
}

int main() {
    std::string message = greet("World");
    std::cout << message << std::endl;
    return 0;
}
