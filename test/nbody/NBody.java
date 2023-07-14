import java.lang.Math; 
class Rand{
    static float state;
    static float Rand(){
        state = ((state*state + .122423255f)*17.200424f - 1.234343f);
        state = state%1.0f;
        return state;
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
      float sqrMag = (this.x*this.x)+(this.y*this.y)+(this.z*this.z);
      return sqrMag;
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
        if (mag < 0.0001 && mag > -0.0001) mag = 1.0f;
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
  final float TIME_SCLAE = 0.001f;
  public Planet(){
      this.mass = 1.0f + Rand.Rand()*Rand.Rand()*2.0f; 
      this.position = Vector3.Random();
      this.velocity = Vector3.Random();
  }
  public void SimulateInteraction(Planet other){
      float distance = Vector3.Distance(this.position,other.position);
      Vector3 diffrence = other.position.clone().Sub(this.position);
      if(distance < 0.00001)return;
      float force = (this.mass + other.mass)/(distance*distance);
      Vector3 direction = diffrence.Normalize();
      this.velocity.Add(direction.Mul(force*TIME_SCLAE).Mul(TIME_SCLAE));
  }
  public void Tick(){
      position.Add(velocity);
  }
  public void Display(){
      System.out.print("p:(");
      System.out.print(this.position.x);
      System.out.print(",");
      System.out.print(this.position.y);
      System.out.print(",");
      System.out.print(this.position.z);
      System.out.print(") v:(");
      System.out.print(this.velocity.x);
      System.out.print(",");
      System.out.print(this.velocity.y);
      System.out.print(",");
      System.out.print(this.velocity.z);
      System.out.println(")");
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
     System.out.println("TICK!");
     for(int i = 0; i < planets.length; i++){
        planets[i].Display();
     }
  }
  public static NBody NewNBody(int pc){
    return new NBody(pc);
  }
  public static void main(String[] args){
    NBody n = new NBody(900);
    for(int i = 0; i < 1000; i++){
      n.Tick();
      if(i%100 == 0)n.Display();
    }
  }
}
