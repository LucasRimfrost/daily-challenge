-- seed.sql
-- Daily challenges from 2026-03-14 to 2026-04-13

INSERT INTO challenges (title, description, difficulty, expected_answer, hint, max_attempts, scheduled_date)
VALUES

-- Week 1
('FizzBuzz Count',
 'Consider all integers from 1 to 100. If a number is divisible by 3, it becomes "Fizz". If divisible by 5, it becomes "Buzz". If divisible by both 3 and 5, it becomes "FizzBuzz". How many times does "FizzBuzz" appear?',
 'easy', '6', 'Think about multiples of 15.', 5, '2026-03-14'),

('Reverse Sum',
 'Take the number 47. Reverse its digits to get 74. Add them together: 47 + 74 = 121. Now do the same for the number 293. What is 293 plus its reverse?',
 'easy', '685', 'The reverse of 293 is 392.', 5, '2026-03-15'),

('Staircase Climb',
 'You have a staircase with 7 steps. You can climb 1 or 2 steps at a time. How many distinct ways can you reach the top?',
 'medium', '21', 'This follows the Fibonacci sequence. Start from step 1 and work your way up.', 5, '2026-03-16'),

('Binary Ones',
 'How many 1s are there in the binary representation of the number 255?',
 'easy', '8', '255 in binary is all ones. How many bits does it take?', 5, '2026-03-17'),

('Missing Number',
 'You have an array containing all integers from 1 to 100, but one number is missing. The sum of the array is 4950. What number is missing?',
 'easy', '50', 'The sum of 1 to 100 is 5050.', 5, '2026-03-18'),

('Matrix Diagonal',
 'Given a 4x4 identity matrix, what is the sum of ALL its elements?',
 'easy', '4', 'An identity matrix has 1s on the diagonal and 0s everywhere else.', 5, '2026-03-19'),

('Palindrome Chain',
 'Start with the number 87. Add it to its reverse (78) to get 165. Then add 165 to its reverse (561) to get 726. Then add 726 to its reverse (627). What number do you get?',
 'medium', '1353', 'Just follow each step carefully. 726 + 627 = ?', 5, '2026-03-20'),

-- Week 2
('Bit Shift',
 'In most programming languages, what is the result of the expression: 1 << 10?',
 'easy', '1024', 'Left shifting by N is the same as multiplying by 2^N.', 5, '2026-03-21'),

('Collatz Steps',
 'The Collatz sequence starts with a number. If even, divide by 2. If odd, multiply by 3 and add 1. Repeat until you reach 1. Starting from 6, how many steps does it take to reach 1? (Do not count the starting number)',
 'medium', '8', 'The sequence is: 6, 3, 10, 5, 16, 8, 4, 2, 1.', 5, '2026-03-22'),

('Array Rotation',
 'You have the array [1, 2, 3, 4, 5]. After rotating it to the right by 2 positions, what is the resulting array? Write your answer as comma-separated values with no spaces.',
 'easy', '4,5,1,2,3', 'The last 2 elements wrap around to the front.', 5, '2026-03-23'),

('Tower of Hanoi',
 'What is the minimum number of moves required to solve the Tower of Hanoi puzzle with 5 disks?',
 'medium', '31', 'The formula is 2^n - 1 where n is the number of disks.', 5, '2026-03-24'),

('XOR Puzzle',
 'What is the result of: 5 XOR 3 XOR 5 XOR 7 XOR 3?',
 'medium', '7', 'XOR of a number with itself is 0. XOR of a number with 0 is itself.', 5, '2026-03-25'),

('Egg Drop',
 'You have 2 eggs and a 100-floor building. You want to find the highest floor from which an egg can be dropped without breaking. What is the minimum number of drops you need in the worst case to guarantee finding the answer?',
 'hard', '14', 'Think about the triangular number approach. With k drops, you can cover k*(k+1)/2 floors.', 5, '2026-03-26'),

('Stack Sequence',
 'You push the numbers 1, 2, 3, 4, 5 onto a stack in order. What is the maximum number of different valid pop sequences possible?',
 'hard', '42', 'This is the 5th Catalan number. The formula is C(2n,n)/(n+1).', 5, '2026-03-27'),

-- Week 3
('Subnet Math',
 'A /24 subnet mask provides how many usable host addresses?',
 'easy', '254', '2^8 = 256 total, minus the network address and broadcast address.', 5, '2026-03-28'),

('Linked List Cycle',
 'A linked list has nodes: 1 -> 2 -> 3 -> 4 -> 5 -> 3 (back to node 3). If you use the slow/fast pointer technique starting at node 1, at which node value do the pointers first meet?',
 'hard', '4', 'Slow moves 1 step, fast moves 2 steps. Trace through each step carefully.', 5, '2026-03-29'),

('Regex Match Count',
 'Given the string "aababcabcabc", how many non-overlapping matches does the pattern "abc" have?',
 'easy', '3', 'Scan left to right, each match consumes those characters.', 5, '2026-03-30'),

('Graph Edges',
 'A complete undirected graph has 6 nodes. How many edges does it have?',
 'medium', '15', 'The formula is n*(n-1)/2.', 5, '2026-03-31'),

('Power of Two',
 'What is the largest power of 2 that is less than 1000?',
 'easy', '512', 'Start doubling: 1, 2, 4, 8, 16, 32, 64, 128, 256, ...', 5, '2026-04-01'),

('Hash Collision',
 'You have a hash table with 365 slots. Using the birthday paradox approximation, roughly how many items do you need to insert before there is a 50% chance of a collision? Round to the nearest whole number.',
 'hard', '23', 'The birthday paradox! With 365 days, you need about 23 people.', 5, '2026-04-02'),

('Sorting Swaps',
 'Using bubble sort on the array [5, 3, 1, 4, 2], how many swaps are needed to fully sort it in ascending order?',
 'medium', '7', 'Walk through each pass of bubble sort and count every swap.', 5, '2026-04-03'),

-- Week 4
('Boolean Logic',
 'What is the result of: (true AND false) OR (NOT false AND true)? Answer with "true" or "false".',
 'easy', 'true', 'Evaluate each part in parentheses first.', 5, '2026-04-04'),

('Heap Height',
 'A complete binary heap contains 100 elements. What is the height of the heap? (Root is height 0)',
 'medium', '6', 'The height is floor(log2(n)). What is floor(log2(100))?', 5, '2026-04-05'),

('Two Sum',
 'Given the array [2, 7, 11, 15] and a target of 9, which two indices (0-based) add up to the target? Answer as two comma-separated numbers, smaller index first.',
 'easy', '0,1', 'Which two numbers in the array sum to 9?', 5, '2026-04-06'),

('Deadlock Conditions',
 'How many conditions must ALL be true simultaneously for a deadlock to occur in a system? (Coffman conditions)',
 'medium', '4', 'Mutual exclusion, hold and wait, no preemption, and...', 5, '2026-04-07'),

('Tree Traversal',
 'Given a binary tree where the inorder traversal is [1,2,3,4,5] and the preorder traversal is [3,2,1,5,4], what is the postorder traversal? Answer as comma-separated values.',
 'hard', '1,2,4,5,3', 'The first element of preorder is the root. Use that to split the inorder array.', 5, '2026-04-08'),

('Cache Lines',
 'If a CPU cache line is 64 bytes and an int is 4 bytes, how many ints fit in a single cache line?',
 'easy', '16', 'Simple division: 64 / 4.', 5, '2026-04-09'),

('Dijkstra Steps',
 'In a graph with vertices A, B, C, D and edges A-B(1), A-C(4), B-C(2), B-D(5), C-D(1), what is the shortest path distance from A to D?',
 'medium', '4', 'A -> B -> C -> D might be shorter than going directly.', 5, '2026-04-10'),

('Mutex Count',
 'The Dining Philosophers problem has 5 philosophers around a table. What is the minimum number of forks that must be available to guarantee that at least one philosopher can eat?',
 'hard', '5', 'Each philosopher needs 2 forks. Think about the worst case where everyone holds one fork.', 5, '2026-04-11'),

('Fibonacci Modulo',
 'What is the 20th Fibonacci number modulo 100? (F(1)=1, F(2)=1, F(3)=2, ...)',
 'medium', '65', 'Compute the sequence up to F(20) = 6765. Then take mod 100.', 5, '2026-04-12'),

('Traveling Salesman',
 'With 4 cities, how many unique round-trip routes exist for the Traveling Salesman Problem? (Starting city is fixed)',
 'medium', '6', 'With a fixed start, you permute the remaining (n-1) cities: (n-1)!', 5, '2026-04-13');