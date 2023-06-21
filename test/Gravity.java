public class Gravity{
    float x;
    float y;
    float vx;
    float vy;
    static float Sqrt(float in){
      return in;
    }
    void Tick(){
       float distance = Sqrt(x*x + y*y);
       float t = 0.01f;
       vx -= (x * distance)*t;
       vy -= (y * distance)*t;
       x += vx*t;
       y += vy*t;
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
