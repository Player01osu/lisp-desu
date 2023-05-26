
print("sugma nuts")
main()
test(nuts [])
defun(main [] 
print("Hello, world!"))
in-package(#:guessing_game)
defun(stdout 
text() 
format(t text) 
finish-output())
defun(print_num 
n() 
stdout(concatenate(' string 
write-to-string(n) "~%")))
defun(loop_hi 
n() 
if(< nil 
progn(print_num(n) 
loop_hi(-(n 1)))))
defun(guess_loop 
num() 
loop(let(guess(parse-integer(read-line()))() 
cond(>(stdout("Guess is too large~%")) 
<(stdout("Guess is too small~%")) 
=(return(0))))))
defun(main [] 
let(num(random(100))() 
stdout("Welcome to le guessing game!~%Guess a number between 1-100.~%") 
guess_loop(num) 
stdout("That's correct!~%")))