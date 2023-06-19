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
  public static int ReturnFirstBackend(int x,int y,int z,int w,int q){
      return x;
  }
  public static int ReturnFirst(int x,int y,int z,int w,int q){
      return ReturnFirstBackend(x,y,z,w,q);
  }
  public static int ReturnLastBackend(int x,int y,int z,int w,int q){
      return q;
  }
  public static int ReturnLast(int x,int y,int z,int w,int q){
      return ReturnLastBackend(x,y,z,w,q);
  }
  public static int ReturnSecondBackend(int x,int y,int z,int w,int q){
      return y;
  }
  public static int ReturnSecond(int x,int y,int z,int w,int q){
      return ReturnSecondBackend(x,y,z,w,q);
  }
  public static int ReturnForthBackend(int x,int y,int z,int w,int q){
      return w;
  }
  public static int ReturnForth(int x,int y,int z,int w,int q){
      return ReturnForthBackend(x,y,z,w,q);
  }
}
