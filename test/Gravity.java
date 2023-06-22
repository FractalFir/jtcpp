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
       //float t = 0.01f;
       float magnitude = Sqrt(x*x + y*y);
       
       vx -= ((x/magnitude) / (distance*distance));//*t;
       vy -= ((y/magnitude) / (distance*distance));//*t;
       x += vx;//*t;
       y += vy;//*t;
    }
    void SetPos(float x, float y){
      this.x = x;
      this.y = y;
    }
    void SetVel(float vx, float vy){
      this.vx = vx;
      this.vy = vy;
    }
    float GetX(){
      return x;
    }
    float GetY(){
      return y;
    }
}
