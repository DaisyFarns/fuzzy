import json
import os
import itertools
import random
import re

import cv2
from skimage.metrics import hausdorff_distance

base64chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz01234567890+/"

def gen_char_images():
    # os.mkdir("chars")

    full_image = cv2.imread("all_chars.png")

    char_size = 20
    offset = 1
    width_offset = 1

    print()

    images = []
    for i in range(64):
        location = offset + i * char_size
        cropped_char = full_image[4:, location : location + 25]
        width = cropped_char.shape[1]
        cropped_char = cropped_char[:, width_offset : width - width_offset]
        grayscale_char_img = cv2.cvtColor(cropped_char, cv2.COLOR_BGR2GRAY)
        blurred_img = cv2.blur(grayscale_char_img, (9, 9))

        images.append(blurred_img)

    A = images[0]
    o = images[14 + 26]
    i = images[8 + 26]
    j = images[9 + 26]

    print(hausdorff_distance(A, o, method="modified"))
    print(hausdorff_distance(i, j, method="modified"))
    print(hausdorff_distance(j, i, method="modified"))
    print(hausdorff_distance(i, i, method="modified"))

    cv2.imshow("A", A)
    cv2.imshow("O", o)
    cv2.imshow("i", i)
    cv2.imshow("j", j)
    cv2.waitKey(0)

    # Okay so this didn't work out, the distance between i and j is larger
    # then A and o

FILEPATH = "../base64similarities.json"

def manual_comparison():
    if not os.path.isfile(FILEPATH):
        print("Creating file")
        f = open(FILEPATH, "w")
        json.dump([[None for _ in range(64)] for _ in range(64)], f)
        f.close()

    with open(FILEPATH) as file:
        similarity = json.load(file)
        for i, j in itertools.product(range(64), repeat=2):
            if similarity[i][j] is not None:
                similarity[i][j] = int(similarity[i][j])

    done = False

    pairs = itertools.combinations(range(64), 2)
    pairs = list(pairs)
    random.shuffle(pairs)

    

    try:
        while not done:
            for i in range(10):
                try:
                    pair = pairs.pop(-1)
                except IndexError:
                    continue

                if similarity[pair[0]][pair[1]] is None:

                    correct = False
                    while not correct:

                        inp = input(
                                f"{base64chars[pair[0]]} - {base64chars[pair[1]]} > "
                            )

                        if re.match(r"^[0-9]$", inp):
                            correct = True

                    similarity[pair[0]][pair[1]] = int(inp)
                    similarity[pair[1]][pair[0]] = int(inp)

                              
            with open(FILEPATH, "w") as file:
                json.dump(similarity, file, indent=4)

            if len(pairs) == 0:
                done = True

    except KeyboardInterrupt:
        with open(FILEPATH, "w") as file:
            json.dump(similarity, file, indent=4)



if __name__ == "__main__":
    manual_comparison()
