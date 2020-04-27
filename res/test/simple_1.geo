
type Point: (int(x), int(y),)

calc start: () {
  // Just a test that takes a single object and tries to print it
  [Point(1,2)] => print;

  // Test Assignment
  //p: [Point(1,2)];
  //p => print;
  //
  // hard code sum first, then try to use the def below
  //sum(p => x, p => y) => print;
  //
  // Let's start using arrays
  // a: [Point(1,2), Point(2,3), Point(3,4)];
  // a => print
  //test lambda (kinda like a map)
  // a => Point(x1, _) => sum(x1, 5) => print;
  //test filter
  // a => Point(x1 < 3, y1) => y1 => print;
}

/*
redeclaration of what print is like

calc print: [..value] {
  value.print
}

trans x: Point(x1, _) {int(x1)}
trans y: Point(_, y1) {int(y1)}

Test sum
trans sum: (a,b) {a + b}

Test summing all the values in a struct with overloading
trans sum: [] {0}
trans sum: [value] {value}
trans sum: [value, next_value] {value + next_value}
*/
