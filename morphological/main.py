# %%
import numpy as np
import cv2
import queue
from matplotlib import pyplot as plt


# %%
def imshow(img):
    plt.imshow(img, cmap="gray")
    plt.axis("off")
    plt.show()


# %%
def dilate_outside(img):
    """将图像背景膨胀，以消除外面那一圈。返回保留的前景掩码（255 表示保留）。

    Args:
        img: 二维的灰度图像。
    """
    is_visited = np.zeros(img.shape, np.bool_)
    background_mask = np.zeros(img.shape, np.uint8)
    q = queue.Queue()
    q.put((0, 0))
    is_visited[0, 0] = True
    while not q.empty():
        x, y = q.get()
        background_mask[x, y] = 255
        for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
            nx, ny = x + dx, y + dy
            if (
                nx >= 0
                and nx < img.shape[0]
                and ny >= 0
                and ny < img.shape[1]
                and not is_visited[nx, ny]
                and np.abs(int(img[nx, ny]) - int(img[0, 0])) <= 16
            ):
                q.put((nx, ny))
                is_visited[nx, ny] = True
    kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (7, 7), anchor=(3, 3))
    background_mask = cv2.dilate(
        background_mask,
        kernel,
        borderType=cv2.BORDER_CONSTANT,
        borderValue=0,
    )

    foreground_mask = cv2.bitwise_not(background_mask)
    kernel = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (33, 33), anchor=(16, 16))
    foreground_mask = cv2.erode(
        foreground_mask,
        kernel,
        borderType=cv2.BORDER_CONSTANT,
        borderValue=0,
    )

    return foreground_mask


def to_binary(img):
    """将图片按照自定义的规则二值化。

    Args:
        img: YUV 格式的图片。
    """
    ret = np.zeros(img.shape[:2], np.uint8)
    for i in range(img.shape[0]):
        for j in range(img.shape[1]):
            p = img[i, j]
            if p[0] > 160 and p[2] > 120:
                ret[i, j] = 255
    return ret


# %%
def main():
    img = cv2.imread("cell.png")
    img = cv2.cvtColor(img, cv2.COLOR_BGR2YUV)
    plt.imsave("gray.png", img[:, :, 0], cmap="gray")
    plt.imsave("u.png", img[:, :, 1], cmap="gray")
    plt.imsave("v.png", img[:, :, 2], cmap="gray")
    imshow(img[:, :, 0])

    mask = dilate_outside(img[:, :, 0])
    img[mask == 0, :] = [0, 128, 128]
    plt.imsave("dilated_outside.png", cv2.cvtColor(img, cv2.COLOR_YUV2RGB))
    imshow(img[:, :, 0])

    img = to_binary(img)
    plt.imsave("binary.png", img, cmap="gray")
    imshow(img)


# %%
if __name__ == "__main__":
    main()
