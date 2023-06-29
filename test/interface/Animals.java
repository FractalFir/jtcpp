interface Animal{
  public void Sound();
  public String Describe();
  default void Feed(){
    System.out.println("Feeding the animal!");
  }
}
class Dog implements Animal{
  public void Sound(){
    System.out.println("Woof!");
  }
  public String Describe(){
      return "This is a doggy doggo!";
  }
}
class Tester{
  static void Main(){
    Dog rex = new Dog();
    rex.Feed();
    rex.Sound();
    //Animal unknown = rex;
  }
}
