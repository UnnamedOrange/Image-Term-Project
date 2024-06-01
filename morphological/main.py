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
    img = cv2.cvtColor(img, cv2.COLOR_BGR2YUV)
    plt.imsave("gray.png", img[:, :, 0], cmap="gray")
    plt.imsave("u.png", img[:, :, 1], cmap="gray")
    plt.imsave("v.png", img[:, :, 2], cmap="gray")
    imshow(img[:, :, 0])


# %%
if __name__ == "__main__":
    main()
