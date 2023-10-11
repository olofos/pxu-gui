# PXU gui

`PXU gui` gives a visualisation of the relation between the variables `p`, `x⁺`, `x⁻` and `u` which are useful for describing the kinematics of world-sheet excitations of the light-cone gauge string in AdS₃ × S³ × T⁴ supported by a mix of RR and NSNS flux.

There are four main panels showing the `p`, `x⁺`, `x⁻` and `u` planes. In each plane there is a background grid which represent the images of `X⁺(p,m)` and `X⁻(p,m)` for integers `m` and real `p`. Furthermore there are black, green and red lines which represent cuts in the various planes.

The state is represented by dots can be dragged around and dragging them through a cut brings the point to a different sheet of the full Riemann surface. The last moved dot is considered active. The dots are blue if they are on the same sheet as the active dot, otherwise they are gray.

On the right hand side there are sliders to pick the values for the coupling constants `h` and `k` as well as the bound state number `M`. Changing the bound state number resets the state to a standard position. There is also a `Reset state` button which can be used to go back to a standard state.

The various planes can be scrolled either by dragging, or by using the scroll wheel (just the scroll wheel scrolls vertically, and with the `Shift` key held down it scrolls vertically). They can also be zoomed in and out using `Ctrl` and the scroll wheel. Most standard touch screen controls work as expected.

By double clicking on one of the main panels, that plane is shown in full screen. To exit just double click again or press `Escape`.

## Cut types

-   Solid red and green cuts represent the "scallion" in the `x⁺` and `x⁻` planes, respectively.
-   Dashed red and green cuts represent the "kidney" in the `x⁺` and `x⁻` planes, respectively.
-   Solid black cuts represent the square root branch cut of the dispersion relation.
-   There is a log cut along the real line in the `x⁺` and `x⁻` planes, which we only indicate on those planes.

Note that we draw both actual cuts and the image of resolved cuts. For example, the solid red line corresponding to the `x⁺` scallion represents an actual branch cut in the `x⁻` and `u` planes, but not in the the `p` and `x⁺` planes.

-   `p` plane: only the black `E(p)` cuts give branch cuts
-   `x⁺` plane: the green (`x⁻`) "scallion" and "kidney" cuts give branch cuts
-   `x⁻` plane: the red (`x⁺`) "scallion" and "kidney" cuts give branch cuts
-   `u` plane: both the red and green (`x⁺` and `x⁻`) "scallion" and "kidney" cuts give branch cuts

## Keyboard shortcuts

-   _Home_: center the view in each plane on the state.
-   _1_ to _9_: construct a state with the corresponding bound state number.
-   _Space_: Lock/unlock bound state.
-   `+` and `-`: Add or remove one excitation. This only works when the bound state is unlocked.
-   _Backspace_: Resets the state. This has the same effect as clicking the `Reset State` button.
-   _R_: Holding down _R_ while dragging in p space makes the dragged point stick to the real line. In u space it instead sticks to a horizontal line with imaginary part a multiple of `i/h`.
-   _E_/_W_: Holding one of these keys down while dragging a point restrict the motion to the horizontal/vertical axis.
-   _Escape_: Exit full screen mode.
-   _Enter_: Hide/show the side panel.
-   _Left_/_Right_: make the previous/next excitation the active excitation.
-   _Up_/_Down_: reorder the excitations. This only works when the bound state is unlocked.

## Known issues

-   Occasionally an excitation ends up in an inconsistent state. The only way to resolve this is to either reset the whole state, or the unlock the bound state and the remove the inconsistent excitation and add it back in.
-   Sometimes dragging a point does not work because the program does not find a suitable numerical solution. This tends to happen when crossing a cut, and in that case crossing a bit further up the cut or with a bit more speed often helps. It is also for similar reasons not possible to drag points in the `u` plane when the coupling constant `h` is below around 0.8.
-   For `k=1` and `k=2` the cut structure is quite different from the generic case of `k > 2`, with, eg, several cuts overlapping. These cases are not very well supported. The pure RR case (`k=0`) is specially handled and works quite well.
