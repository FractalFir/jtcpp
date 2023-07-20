// Interfaces don't yet work!
interface Animal{
  public void Sound();
}
class Dog implements Animal{
  public void Sound(){
    System.out.println("Woof!");
  }
}
class Cow implements Animal{
  public void Sound(){
    System.out.println("Moo!");
  }
}
class Duck implements Animal{
  public void Sound(){
    System.out.println("Quack!");
  }
}
