public class Gravity{
    float x;
    float y;
    float vx;
    float vy;
    float Sqrt(float in){
      return in;
    }
    void Tick(){
       float distance = Sqrt(x*x + y*y);
       vx -= x * distance;
       vy -= y * distance;
       x += vx;
       y += vy;
    }
    void Set(float x, float y){
      this.x = x;
      this.y = y;
    }
    float GetX(){
      return x;
    }
    float GetY(){
      return y;
    }
}
