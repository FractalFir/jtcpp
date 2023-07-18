public class cat {
    public static void main (String[] args) {
        int argnum = 0;
        for (String s: args) {
            System.out.print("Arg number ");
            System.out.print(argnum);
            System.out.print(" is \"");
            System.out.print(s);
            System.out.println("\"");
            argnum += 1;
        }
    }
}
