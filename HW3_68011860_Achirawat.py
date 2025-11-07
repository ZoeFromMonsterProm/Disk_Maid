name = input("Enter employee's name: ")
hours_work = float(input("Enter number of hours worked in a week: "))
hourly_pay = float(input("Enter hourly pay rate: "))
federal_tax = float(input("Enter federal tax withholding rate: "))
state_tax = float(input("Enter state tax withholding rate: "))

gross_pay = hours_work * hourly_pay
federal_withholding = gross_pay * federal_tax
state_withholding = gross_pay * state_tax
total_deduction = federal_withholding + state_withholding
net_pay = gross_pay - total_deduction

print("Employee Name:", name)
print("Hours Worked: {:.1f}".format(hours_work))
print("Pay Rate: ${:.2f}".format(hourly_pay))
print("Gross Pay: ${:.2f}".format(gross_pay))
print("Deductions:")
print(" Federal Withholding ({:.1f}%): ${:.2f}".format(federal_tax * 100, federal_withholding))
print(" State Withholding ({:.1f}%): ${:.2f}".format(state_tax * 100, state_withholding))
print(" Total Deduction: ${:.2f}".format(total_deduction))
print("Net Pay: ${:.2f}".format(net_pay))



number = input("Enter a four-digit integer: ")
if len(number) == 4 and number.isdigit():
    print("Reversed number:", number[::-1])
else:
    print("Please enter a four-digit integer")



import turtle
radius = float(input("Please enter the radius: "))
colors = ["blue", "black", "red", "yellow", "green"]
positions = [(-220, 0), (-75, 0), (70, 0), (-150, -80), (-5, -80)]
t = turtle.Turtle()
t.pensize(5)
for pos, color in zip(positions, colors):
    t.penup()
    t.goto(pos)
    t.pendown()
    t.color(color)
    t.circle(radius)
turtle.done()



import turtle

length = float(input("Enter the length of the star: "))
t = turtle.Turtle()

for i in range(5):
    t.forward(length) 
    t.right(144)
turtle.done()



x1, y1 = map(float, input("Enter x1 y1: ").split())
x2, y2 = map(float, input("Enter x2 y2: ").split())
x3, y3 = map(float, input("Enter x3 y3: ").split())

import turtle
t = turtle.Turtle()
t.penup()
t.goto(x1, y1)
t.pendown()
t.goto(x2, y2)
t.goto(x3, y3)
t.goto(x1, y1)

area = abs((x1*(y2-y3) + x2*(y3-y1) + x3*(y1-y2))/2)

low_y = min(y1,y2,y3)
mid_x = (x1+x2+x3)/3
t.penup()
t.goto(mid_x, low_y - 30)
t.write("Area = {:.2f}".format(area), align="center")
turtle.done()

if left2 >= left1 and right2 <= right1 and top2 <= top1 and bottom2 >= bottom1:
    result = "Rectangle 2 is inside rectangle 1"
elif left1 >= left2 and right1 <= right2 and top1 <= top2 and bottom1 >= bottom2:
    result = "Rectangle 1 is inside rectangle 2"
elif ((left1<=left2 and right1>=left2) or (right1>=right2 and left1<=right2) or (top1>=top2 and bottom1<=top2) or (bottom1<=bottom2 and top1>=bottom2)):
    result = "Rectangles overlap"
else:
    result = "Rectangle not overlap"