# %%
import numpy as np
import cv2
from matplotlib import pyplot as plt


# %%
def imshow(img):
    plt.imshow(img, cmap="gray")
    plt.axis("off")
    plt.show()


# %%
def main():
    img = cv2.imread("cell.png")
    img = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
    imshow(img)


# %%
if __name__ == "__main__":
    main()
