public class Sieve{
    static boolean[] numbers;
    static void Init(int max_n){
        numbers = new boolean[max_n];
        for(int i = 0; i < max_n; i++){
            numbers[i] = true;
        }
    }
    static void Mark(int curr_prime){
        
    }
    static void Run(){  
        for(int curr_prime = 2; curr_prime < numbers.length; curr_prime++){
          for(int i = curr_prime*2; i < numbers.length; i+= curr_prime){
              numbers[i] = false; 
          }
          Mark(curr_prime);
        }
    }
    static void Display(){
          for(int i = 2; i < numbers.length; i++){
              if(numbers[i]){
                  System.out.println(i);
              }
          }
    }
    public static void main(String[] args){
      Init(100000000);
      Run();
      Display();
    }
}
