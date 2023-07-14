class State{
    int NextStateIf0;
    int NextStateIf1;
    byte MoveIf0;   
    byte MoveIf1;
    boolean TapeSetIf0;
    boolean TapeSetIf1;
}
class TuringMachine{
    State[] states;
    boolean[] tape;
    int currState;
    int head;
    public TuringMachine(State[] states, int length){
        this.states = states;
        this.tape = new boolean[length];
        currState = 0;
        head = length/2;
    }
    void PrintState(){
        System.out.print("|If\t|NextState\t|Move\t|TapeSet\t|\n");
        System.out.print("|0 \t|");
        System.out.print(states[currState].NextStateIf0);
        System.out.print("\t\t|");
        System.out.print(states[currState].MoveIf0);
        System.out.print("\t|");
        System.out.print(states[currState].TapeSetIf0);
        System.out.print("|\n");
        System.out.print("|1 \t|");
        System.out.print(states[currState].NextStateIf1);
        System.out.print("\t\t|");
        System.out.print(states[currState].MoveIf1);
        System.out.print("\t|");
        System.out.print(states[currState].TapeSetIf1);
        System.out.println();
    }
    void PrintHead(){
        for(int i = 0; i < head; i++){
            System.out.print("   ");
        }
        System.out.print(" ↓ ");
        for(int i = 0; i < tape.length - head; i++){
            System.out.print("   ");
        }
        System.out.println();
    }
    void PrintTape(){
        for(int i = 0; i < tape.length; i++){
            System.out.print('[');
            if(tape[i]){
                System.out.print('1');
            }
            else{
                System.out.print('0');
            }
            System.out.print(']');
        }
        System.out.println();
    }
    void PrintCurrent(){
        PrintState();
        PrintHead();
        PrintTape();
    }
    public boolean Tick(){
        State current = states[currState];
        PrintCurrent();
        int nextState;
        boolean tapeValue = tape[head];
        if(tapeValue){
            nextState = current.NextStateIf1;
        }
        else{
            nextState = current.NextStateIf0;
        }
        if(nextState >= states.length){
            System.out.println("HALT!");
            return false;
        }
        int nextHeadPosition = head;
        if(tapeValue){
            nextHeadPosition += current.MoveIf1;
        }
        else{
            nextHeadPosition += current.MoveIf0;
        }
        if(tapeValue){
            tape[head] = current.TapeSetIf1;
        }
        else{
            tape[head] = current.TapeSetIf0;
        }
        currState = nextState;
        head = nextHeadPosition;
        return true;
    }
    public static void main(String[] args){
        State[] states = new State[2];
        states[0] = new State();
        states[0].NextStateIf0 = 1;
        states[0].NextStateIf1 = 1;
        states[0].MoveIf0 = 1;
        states[0].MoveIf1 = -1;
        states[0].TapeSetIf0 = true;
        states[0].TapeSetIf1 = true;
        states[1] = new State();
        states[1].NextStateIf0 = 0;
        states[1].NextStateIf1 = 2;
        states[1].MoveIf0 = -1;
        states[1].MoveIf1 = 1;
        states[1].TapeSetIf0 = true;
        states[1].TapeSetIf1 = true;
        TuringMachine machine = new TuringMachine(states,20);
        while(machine.Tick()){}
    }
}