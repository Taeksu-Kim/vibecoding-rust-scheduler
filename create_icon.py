from PIL import Image, ImageDraw, ImageFont
import os

# Create a 1024x1024 image with green background
size = 1024
img = Image.new('RGB', (size, size), color='#22c55e')
draw = ImageDraw.Draw(img)

# Draw a simple calendar icon
# White rounded rectangle
padding = 150
rect_coords = [padding, padding, size - padding, size - padding]
draw.rounded_rectangle(rect_coords, radius=80, fill='white')

# Green header bar
header_height = 200
draw.rounded_rectangle(
    [padding, padding, size - padding, padding + header_height],
    radius=80,
    fill='#16a34a'
)

# Draw text "S" for Scheduler
try:
    # Try to use a system font
    font = ImageFont.truetype("arial.ttf", 400)
except:
    # Fallback to default font
    font = ImageFont.load_default()

text = "S"
# Get text bounding box
bbox = draw.textbbox((0, 0), text, font=font)
text_width = bbox[2] - bbox[0]
text_height = bbox[3] - bbox[1]

# Center the text
x = (size - text_width) / 2
y = (size - text_height) / 2 + 50

draw.text((x, y), text, fill='#22c55e', font=font)

# Save the image
output_path = os.path.join('src-tauri', 'app-icon.png')
img.save(output_path)
print(f"Icon created at: {output_path}")
