public class Calls{
  public static int Square(int a){
    return a*a;
  }
  public static int Sum(int x,int y, int z){
      return x + y + z;
  }
  public static int SqrMag(int x,int y,int z){
      int xx = Square(x);
      int yy = Square(y);
      int zz = Square(z);
      return Sum(xx,yy,zz);
  }
}
