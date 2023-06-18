public class BasicArthm{
  public static int Add(int a,int b){
      return a + b;
  }
  public static int Sub(int a, int b){
      return a - b;
  }
  public static int Mul(int a, int b){
      return a * b;
  }
  public static int Div(int a, int b){
      return a / b;
  }
  public static int Mod(int a, int b){
      return a % b;
  }
  public static int MultiOp(int a, int b){
    int sum = a + b;
    int mul = a * b;
    int dif = sum - mul;
    return ((dif%sum) + mul)/mul;
  }
}
