// 📖 # This is a C# example file demonstrating basic syntax and functionality

using System;

public class Example
{
    public static void Main(string[] args)
    {
        string message = Greet("World");
        Console.WriteLine(message);
    }
    
    public static string Greet(string name)
    {
        return $"Hello, {name}!";
    }
}
