# zsp
This is the code for a toy language I built because I was bored.

The documentation for the language and guide is on the wiki, here is just a basic summary of the language and it's features

ZSP stands for "ZSP is Superior to Python", because I wanted one of those fancy recursive acryonyms and I couldn't come up with a better name

 
## getting started
You can download the interpreter and use all the features, or if you just want to play around, there's a [web interpreter](https://coolelectronics.me/playground/) available

If you decide to use the native interpreter, it will create a .zsp folder in your home directory where libraries will be loaded upon first run. If you want to use a library such as the `fs` or `dlopen` libraries, you will need to put the shared object there.

## features
supported:  
  dynamic typing  
  functions  
  variables  
  loading shared object libraries   
  objects  
  arrays  
  for loops  
  

to be implemented:  
  closures  
  dlopen function  
  if else  
## syntax
The syntax I ended up choosing is a mix of c and python, with some ideas stolen from various languages, mostly rust

for an example, here's fizzbuzz in zsp
```
mod a b{
    r = a
    loop{
        r = r - b
        if r < b{
            return r
        }
    }
}
for counter 0 counter < 100 counter = counter + 1{
    by3 = (mod counter 3) == 0
    by5 = (mod counter 5) == 0

    if by3 && by5 {
        put "FizzBuzz"
    }else{
        if by3 && (by5 == false){
            put "Fizz"
        }
        if by5 && (by3 == false){
            put "Buzz"
        }
        if (by5 == false) && (by3 == false){
            put counter
        }
    }
}
```

more information about the specifics are on the wiki
