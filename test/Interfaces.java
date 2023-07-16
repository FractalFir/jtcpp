interface Animal{
  public void Sound();
}
class Dog implements Animal{
  public void Sound(){
    System.out.println("Woof!");
  }
}
class Cat implements Animal{
  public void Sound(){
    System.out.println("Meow!");
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
