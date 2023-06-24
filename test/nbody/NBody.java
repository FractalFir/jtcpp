import java.lang.Math; 
class Rand{
    static float Rand(){
      return 0.232423f;
    }
}
final class Vector3{
    protected float x;
    protected float y;
    protected float z;
    public Vector3(){
        this.x = 0.0f;
        this.y = 0.0f;
        this.z = 0.0f;
    }
    public Vector3(float x,float y,float z){
        this.x = x;
        this.y = y;
        this.z = z;
    }
    public float SqrMag(){
      return this.x*this.x+this.y*this.y+this.z*this.z;
    }
    public float Magnitude(){
      return (float)Math.sqrt(SqrMag());
    }
    public Vector3 clone(){
      return new Vector3(x,y,z);
    }
    public static float Distance(Vector3 a, Vector3 b){
        Vector3 diff = a.clone().Sub(b);
        return diff.Magnitude();
    }
    public Vector3 Add(Vector3 other){
        this.x += other.x;
        this.y += other.y;
        this.z += other.z;
        return this;
    }
    public Vector3 Sub(Vector3 other){
        this.x -= other.x;
        this.y -= other.y;
        this.z -= other.z;
        return this;
    }
    public Vector3 Normalize(){
        float mag = Magnitude();
        this.x /= mag;
        this.y /= mag;
        this.z /= mag;
        return this;
    }
    public Vector3 Mul(float factor){
      this.x *= factor;
      this.y *= factor;
      this.y *= factor;
      return this;
    }
    public static Vector3 Random(){
        return new Vector3(Rand.Rand(),Rand.Rand(),Rand.Rand());
    }
}
class Planet{
  Vector3 position;
  Vector3 velocity;
  float mass = 1.0f;
  public Planet(){
      this.mass = 1.0f + Rand.Rand()*Rand.Rand()*2.0f; 
      this.position = Vector3.Random();
      this.velocity = Vector3.Random();
  }
  public void SimulateInteraction(Planet other){
      float distance = Vector3.Distance(this.position,other.position);
      Vector3 diffrence = other.position.clone().Sub(this.position);
      float force = (this.mass + other.mass)/(distance*distance);
      Vector3 direction = diffrence.Normalize();
      this.velocity.Add(direction.Mul(force));
  }
  public void Tick(){
      position.Add(velocity);
  }
  public void Display(){
      //TODO:Display data!
  }
}
class NBody{
  Planet[] planets;
  public NBody(int planetCount){
      planets = new Planet[planetCount];
      for(int i = 0; i < planetCount; i++){
        planets[i] = new Planet();
      } 
  }
  public void Tick(){
     for(int i = 0; i < planets.length; i++){
        for(int j = 0; j < planets.length; j++){
            if(i == j) continue;
            planets[i].SimulateInteraction(planets[j]);
        }
     }
     for(int i = 0; i < planets.length; i++){
        planets[i].Tick();
     }
  }
  public void Display(){
     for(int i = 0; i < planets.length; i++){
        planets[i].Display();
     }
  }
  public static NBody NewNBody(int pc){
    return new NBody(pc);
  }
}