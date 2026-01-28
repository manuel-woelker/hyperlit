// 📖 # This is a Go example file demonstrating basic syntax and functionality

package main

import "fmt"

func greet(name string) string {
	return "Hello, " + name + "!"
}

func main() {
	message := greet("World")
	fmt.Println(message)
}
