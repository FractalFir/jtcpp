// This example uses lambdas, which use `InvokeDynamic` op under the hood.
import java.util.List;
class InvokeDynamic { 
    public static void main(String[] args) {
        long lengthyColors = List.of("Red", "Green", "Blue")
          .stream().filter(c -> c.length() > 3).count();
    }
}
