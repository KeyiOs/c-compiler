int main() {
    struct Person {
        char name[50];
        int age;
        float height;
    } p1;

    struct Person p1 = {"Alice", 30, 5.5}, *p2 = {"Alice", 30, 5.5}, p3[5] = {{"Bob", 25, 6.0}, {"Charlie", 28, 5.8}};

    return 0;
}
                            
                        
